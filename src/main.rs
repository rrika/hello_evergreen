extern crate num;
#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate nix;
extern crate libc;
extern crate image;
extern crate getopts;

extern crate xcb;
#[macro_use]
extern crate wayland_client;
extern crate wayland_protocols;

mod cs;
#[macro_use]
mod display;
mod drm_radeon_ioctl;
mod initseq;
mod libdrm;
mod pm4;
mod r600_pci_ids;

use cs::*;
use drm_radeon_ioctl::*;
use initseq::INITSEQ;
use std::env;
use std::fs;
use std::os::unix::io::AsRawFd;
use display::*;
use getopts::Options;

#[repr(C)]
pub struct BOLayout {
	pub cb: [u8; L_CB_SIZE],
	pub db: [u8; L_DB_SIZE],
	pub sh: [u8; L_SHADERBLOB_SIZE],
	pub vx: [f32; L_VERTEXBUFFER_SIZE/4],
	pub timestamps: [u64; 4],
	pub align_to_256: [u8; 256-8*4-L_VERTEXBUFFER_SIZE],
	pub cl: [f32; 4]
}

macro_rules! offset_of {
	($t:ty => $m:ident) => (unsafe {
		let np = 0 as *const $t;
		let mp = (&(*np).$m) as *const _;
		mp as usize
	})
}


const THEDRAW: DrawInfo<'static> = DrawInfo {
	indirect: false,
	indexed: true,
	index_size: 4,
	instance_count: 1,
	user_buffer: Some(&[0, 2, 1, 1, 2, 3])
};

fn bomap(fd: i32, handle: u32, offset: u64, size: u64) -> Mapping {
	let mut mmap_args = DrmRadeonGemMmap {
		handle: handle,
		pad: 0,
		offset: offset,
		size: size,
		addr_ptr: 0
	};
	let addr;
	unsafe {
		/*let r =*/
		drm_ioctl_radeon_gem_mmap(fd, &mut mmap_args);
		// println!("mmap ioctl -> {}", r);
		// println!("{:?}", std::io::Error::last_os_error());
		addr = libc::mmap(0 as *mut libc::c_void, mmap_args.size as usize, libc::PROT_READ|libc::PROT_WRITE, libc::MAP_SHARED, fd, mmap_args.addr_ptr as i64);
	};
	// println!("{:?}", std::io::Error::last_os_error());
	Mapping {ptr: addr, size: mmap_args.size as usize}
}

struct Mapping {
	pub ptr: *mut libc::c_void,
	size: usize
}

impl Drop for Mapping {
	fn drop(&mut self) {
		unsafe { libc::munmap(self.ptr, self.size); }
	}
}

struct BO {
	pub handle: u32,
	pub size: u64,
	fd: i32
}

impl Drop for BO {
	fn drop(&mut self) {
		gem_close(self.fd, self.handle);
	}
}

fn gem_create(fd: i32, size: u64, domain: u32) -> BO {
	let mut create = DrmRadeonGemCreate {
		size: size,
		alignment: 0,
		handle: 0,
		initial_domain: domain,
		flags: 0
	};
	unsafe { drm_ioctl_radeon_gem_create(fd, &mut create) };
	BO {
		handle: create.handle,
		size: create.size,
		fd: fd
	}
}

fn gem_close(fd: i32, handle: u32) {
	let close = DrmGemClose {
		handle: handle,
		pad: 0
	};
	unsafe { drm_ioctl_gem_close(fd, &close) };
}

const BO_DOMAIN: u32 = RADEON_GEM_DOMAIN_VRAM;

fn setup_shaders(cs: &mut CS, bo_reloc: &Fn(&mut CS) -> ()) {
	let vs_conf = ShaderConfig {
		shader_addr  : (offset_of!(BOLayout=>sh) + SH_SOLID_VS_OFFSET) as u32,
		shader_size  : 512,
		num_gprs     : 2,
		stack_size   : 0,
		clamp_consts : 0, // N/A
		export_mode  : 0  // N/A
	};
	evergreen_vs_setup(cs, &vs_conf, bo_reloc);

	let ps_conf = ShaderConfig {
		shader_addr  : (offset_of!(BOLayout=>sh) + SH_SOLID_PS_OFFSET) as u32,
		shader_size  : 512,
		num_gprs     : 1,
		stack_size   : 0,
		clamp_consts : 0,
		export_mode  : 2,
	};
	evergreen_ps_setup(cs, &ps_conf, bo_reloc);
}

fn write_number(cs: &mut CS, number: u64) {
	// Fence, write 64-bit data.
	let offset = unsafe { (&(*(0 as *const BOLayout)).timestamps[0]) as *const u64 as usize };
	// or offset_of!(BOLayout=>timestamps) as u32

	cs.write_label("write number");
	//cs.write(&[ /* set r4 within ME to right value */
	//      packet3(cs::Packet3::DRAW_INDIRECT, 0u32, 0),
	//      ]);
	cs.write(&[
		packet3(cs::Packet3::EVENT_WRITE_EOP, 4u32, 0),
		CACHE_FLUSH_AND_INV_EVENT_TS | event_index(5), //
		//EVENT_TYPE(40/*eop*/) | EVENT_INDEX(0xc),
		/*0u32*/(offset as u32) & !0x3u32, // lower 32 bits of address
		data_sel(2) | int_sel(0) | ((0u64 >> 32) as u32 & 0xffu32), // upper 32-39
		number as u32,
		(number >> 32) as u32,
		/*packet3(cs::Packet3::NOP, 0, 0), /* use the two extra dwords to realign */
		0*/
	]);
}

const SHADERBIN: &'static [u8; 4096] = include_bytes!("../evergreen_shader.bin");

fn init_bo(bo: &mut BOLayout) {
	// let mut f = fs::File::open("evergreen_shader.bin").unwrap();
	// f.read_exact(&mut bo.sh).unwrap();
	bo.sh = *SHADERBIN;

	bo.timestamps[0] = 0xcdcdcdcdcdcdcdcd;
	bo.timestamps[1] = 0xc1c1c1c1c1c1c1c1;
	bo.timestamps[2] = 0xc2c2c2c2c2c2c2c2;
	bo.timestamps[3] = 0xc3c3c3c3c3c3c3c3;

	bo.vx[ 0] = 10.0;
	bo.vx[ 1] = 10.0;
	bo.vx[ 2] = 10.0;
	bo.vx[ 3] = 1.0;

	bo.vx[ 4] = 10.0;
	bo.vx[ 5] = 90.0;
	bo.vx[ 6] = 10.0;
	bo.vx[ 7] = 1.0;

	bo.vx[ 8] = 90.0;
	bo.vx[ 9] = 10.0;
	bo.vx[10] = 10.0;
	bo.vx[11] = 1.0;

	bo.vx[12] = 90.0;
	bo.vx[13] = 90.0;
	bo.vx[14] = 10.0;
	bo.vx[15] = 1.0;

	bo.cl[0] = 0.0;
	bo.cl[1] = 1.0;
	bo.cl[2] = 0.0;
	bo.cl[3] = 1.0;
}

fn build_cs(bo_handle: u32, initseq: &[u32]) -> CS{

	let mut cs = CS::default();

	let bo_reloc = |cs: &mut CS| {
		cs.write_label("  reloc nop");
		cs.write(&[packet3(cs::Packet3::NOP, 0, 0), 0x00000000]);
		cs.write_reloc(bo_handle, 0, BO_DOMAIN, 0);
	};

	write_number(&mut cs, 1);
	bo_reloc(&mut cs);

		cs.write_label("run radeondemo init sequence");
		//initseq_send(&mut cs);
		cs.write(initseq);

		cs.write_label("setup scissors");
		setup_scissors(&mut cs, W, H);

	write_number(&mut cs, 2);
	bo_reloc(&mut cs);

		setup_depth(&mut cs);

	write_number(&mut cs, 3);
	bo_reloc(&mut cs);

		setup_shaders(&mut cs, &bo_reloc);

	write_number(&mut cs, 4);
	bo_reloc(&mut cs);

		cs.write_label("setup framebuffer");
		setup_fb(&mut cs, W, H, TILED, &bo_reloc);

	write_number(&mut cs, 5);
	bo_reloc(&mut cs);

		setup_spi(&mut cs);

	write_number(&mut cs, 6);
	bo_reloc(&mut cs);

		let vtxres = VtxRes {
			byteoffset: offset_of!(BOLayout=>vx) as u32,
			bytesize:   4 * 4 * 4,
			stride:     4 * 4,
			vtxcount:   4
		};
		set_ps_const_buffer(&mut cs, offset_of!(BOLayout=>cl) as u32, &bo_reloc);

	write_number(&mut cs, 7);
	bo_reloc(&mut cs);

		set_vtx_resource(&mut cs, &vtxres, &bo_reloc);
		bo_reloc(&mut cs);

	write_number(&mut cs, 8);
	bo_reloc(&mut cs);

		vbo(&mut cs, THEDRAW);
		cs.write_label("end");

	cs
}

fn render(fd: i32, bo_handle: u32, bo_size: u64, initseq: &[u32]) {
	let mut waitidle = DrmRadeonGemWaitIdle::default();
	waitidle.handle = bo_handle;

	{
		// println!("BO handle = {:?}  size = {:?}", bo_handle, bo_size);
		let mapping = bomap(fd, bo_handle, 0, bo_size);
		let p = mapping.ptr;

		//println!("p = {:?}", p);

		let bo = unsafe { &mut *(p as *mut BOLayout) };
		init_bo(bo);

		//println!("BO unmapped");
	}

	unsafe { drm_ioctl_radeon_gem_wait_idle(fd, &mut waitidle) }; // println!("BO waited");

	let cs = build_cs(bo_handle, initseq);
	cs.submit(fd);
	//println!("CS submitted");

	unsafe { drm_ioctl_radeon_gem_wait_idle(fd, &mut waitidle) }; // println!("BO waited");

	let mut busy = DrmRadeonGemBusy::default();
	busy.handle = bo_handle;
	loop {
		unsafe { drm_ioctl_radeon_gem_busy(fd, &mut busy) };
		// println!("{:?}", busy.domain);
		::std::thread::sleep(::std::time::Duration::new(0, 1));
		if busy.domain == 0 { break }
		break
	}
	//println!("BO is idle");
}

enum Backend { Xcb, Wayland, Kms }
fn backend_from_str(n: &str) -> Option<Backend> {
	use Backend::*;
	match n {
		"xcb" => Some(Xcb),
		"wayland" => Some(Wayland),
		"kms" => Some(Kms),
		_ => None
	}
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {

	let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

	let mut opts = Options::new();
	opts.optflag("h", "help", "print this help menu");
	opts.optopt("d", "dev", "device to use", "DEVICE");
	opts.optopt("o", "write-image", "write colorbuffer to file", "FILENAME");
	opts.optopt("b", "backend", "one of ‘xcb’, ‘wayland’ or ‘kms’", "BACKEND");
	opts.optopt("r", "resolution", "eg. ‘640x480’ (TODO)", "RES");
	opts.optflag("", "info", "display results of gem info and radeon info ioctls");
	opts.optflag("", "minimize-init-seq", "repeatedly run to find necessary packets");

	let matches = match opts.parse(&args[1..]) {
		Ok(m) => { m }
		Err(f) => { panic!(f.to_string()) }
	};

	if matches.opt_present("help") {
		print_usage(&program, opts);
		return
	}

	let backend = if let Some(backend_str) = matches.opt_str("backend") {
		backend_from_str(backend_str.as_str()).expect("unrecognized backend")
	}
	else if let Ok(_) = wayland_client::default_connect() { Backend::Wayland }
	else if let Ok(_) = xcb::Connection::connect(None) { Backend::Xcb }
	else { Backend::Kms };

	let dev_path = if let Some(path) = matches.opt_str("dev") { path }
	else { match backend {
		Backend::Wayland => "/dev/dri/renderD128".to_owned(),
		Backend::Xcb => "/dev/dri/renderD128".to_owned(),
		Backend::Kms => "/dev/dri/card0".to_owned(),
	}};

	println!("Using device {}", dev_path);
	let mut fo_ = fs::OpenOptions::new();
	let fo = fo_.read(true).write(true);

	if matches.opt_present("info") {

		let f = fo.open(dev_path).unwrap();
		let fd = f.as_raw_fd();

		let mut info = DrmRadeonGemInfo::default();
		unsafe { drm_ioctl_radeon_gem_info(fd, &mut info) };
		println!("GART size = {}", info.gart_size);
		println!("VRAM size = {}", info.vram_size);
		println!("VRAM visible = {}", info.vram_visible);

		let mut device_id_64: u64 = 0;
		let mut info = DrmRadeonInfo { request: 0, pad: 0, value: &mut device_id_64 as *mut u64 as u64 };
		unsafe { drm_ioctl_radeon_info(fd, &mut info) };
		let device_id = device_id_64 as u16;
		let (name, family) = r600_pci_ids::pci_id_lookup(device_id).unwrap_or(("unknown", r600_pci_ids::RadeonFamily::UNKNOWN));
		println!("This is a {:?} {:?} chip with PCI device id = {}",
			family,
			name,
			device_id);
		return

	} else if matches.opt_present("minimize-init-seq") {
		let packets = pm4::split(&mut INITSEQ.iter().map(|a|*a));
		let mut mask = packets.iter().map(|_| true).collect::<Vec<_>>();
		for i in 0..packets.len() {

			if packets[i].header & 0xff00 == 0x1000 { continue } // don't toggle nops individually
			let i2 = if i+1<packets.len() && packets[i+1].header & 0xff00 == 0x1000 {
				i+2
			} else {
				i+1
			};

			for m in &mut mask[i..i2] { *m = false; }
			//mask[i] = false;

			let mut compact_stream: Vec<u32> = Vec::new();
			for j in 0..packets.len() {
				if mask[j] {
					print!("x");
					compact_stream.push(packets[j].header);
					compact_stream.extend(packets[j].words.iter().cloned());
				} else {
					print!(" ");
				}
			}
			println!("");

			let f = fo.open(dev_path.clone()).unwrap();
			let fd = f.as_raw_fd();
			let bo = gem_create(fd, std::mem::size_of::<BOLayout>() as u64, BO_DOMAIN);

			{
				let mapping = bomap(fd, bo.handle, 0, bo.size);
				let bo_data = unsafe {&mut (*(mapping.ptr as *mut BOLayout))};
				for pixel in bo_data.cb.iter_mut() { *pixel = 0; }
			}

			let now = std::time::SystemTime::now();
			render(fd, bo.handle, bo.size, &compact_stream);

			{
				let mapping = bomap(fd, bo.handle, 0, bo.size);
				let bo_data = unsafe {&(*(mapping.ptr as *const BOLayout))};
				if true {
					let out = format!("minimize{}.png", i);
					image::save_buffer(&std::path::Path::new(&out), &bo_data.cb, W, H, image::RGBA(8)).unwrap();
				}
				let mut fail = false;
				if now.elapsed().expect("").as_secs() > 1 {
					//println!("fail by timeout");
					fail = true;
				} else {
					for y in 10..90 {
						for x in 10..90 {
							let offset = ((x + y*W)*4) as usize;
							if bo_data.cb[offset..offset+3] != [0, 255, 0] {
								fail = true;
								//println!("fail by pixel difference");
								break
							}
						}
						if fail { break; }
					}
				}
				if fail {
					for m in &mut mask[i..i2] { *m = true; }
					//mask[i] = true;
				}
			}
		}

		for i in 0..packets.len() {
			println!("mask[{}] = {:?}", i, mask[i])
		}

	} else {

		let f = fo.open(dev_path).unwrap();
		let fd = f.as_raw_fd();
		let bo = gem_create(fd, std::mem::size_of::<BOLayout>() as u64, BO_DOMAIN);
		render(fd, bo.handle, bo.size, &[]);

		{
			let mapping = bomap(fd, bo.handle, 0, bo.size);
			let bo_data = unsafe {&(*(mapping.ptr as *const BOLayout))};

			println!("BO dump: {:016x}", bo_data.timestamps[0]);

			if let Some(path) = matches.opt_str("o") {
				image::save_buffer(&std::path::Path::new(path.as_str()), &bo_data.cb, W, H, image::RGBA(8)).unwrap();
			}

			match backend {
				Backend::Wayland => waylandmain(fd, bo.handle),
				Backend::Xcb => xcbmain(fd, bo.handle),
				Backend::Kms => kmsmain(fd, bo.handle)
			};
		}
	}
}
