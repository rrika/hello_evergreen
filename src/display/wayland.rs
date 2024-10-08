use std;
use super::*;
use drm_radeon_ioctl::*;

use wayland_client;
use wayland_client::{Display, EventQueue, GlobalManager, NewProxy, Proxy};
use wayland_client::protocol::{wl_surface, wl_shm_pool, wl_buffer, wl_compositor, wl_shell,
							   wl_subcompositor, wl_shm, wl_shell_surface, wl_seat};

use wayland_protocols;
use wayland_protocols::unstable::linux_dmabuf::v1::client::zwp_linux_dmabuf_v1;
use wayland_protocols::unstable::linux_dmabuf::v1::client::zwp_linux_buffer_params_v1;

struct HelloRadeonWindow {
	s: wl_surface::WlSurface,
	buf: Option<wl_buffer::WlBuffer>,
}

impl<'a> zwp_linux_buffer_params_v1::EventHandler for HelloRadeonWindow {
    fn created(
        &mut self,
        object: zwp_linux_buffer_params_v1::ZwpLinuxBufferParamsV1,
        buffer: NewProxy<wl_buffer::WlBuffer>
    ) {
		println!("created buf");
    	let buffer = buffer.implement_dummy();
		self.s.attach(Some(&buffer), 0, 0);
		self.s.commit();
		self.buf = Some(buffer);
    }

    fn failed(&mut self, object: zwp_linux_buffer_params_v1::ZwpLinuxBufferParamsV1) {
		panic!("wayland buffer sharing failed");	
    }
}


pub fn waylandmain(fd: i32, handle: u32) -> bool {
	let (display, mut event_queue) = match Display::connect_to_env() {
		Ok(ret) => ret,
		Err(e) => return false
	};

	let display = display;

	//let envid = event_queue.add_handler(EnvHandler::<WaylandEnv>::new());
	let globals = GlobalManager::new(&display);

	// prepare the decorated surface
	let (dmabufid, shell_surface, buffer_params) = {

		event_queue.sync_roundtrip().unwrap();

		println!("Globals advertised by server:");
		for (name, ref interface, version) in globals.list() {
			println!("{:4} : {} (version {})", name, interface, version);
		}

		let dmabufid = 0; //state.add_handler(&env.linux_dmabuf);

		let compositor = globals
			.instantiate_exact::<wl_compositor::WlCompositor, _>(1, |compositor| compositor.implement_dummy())
			.unwrap();
		let surface = compositor.create_surface(|surface| surface.implement_dummy())
			.unwrap();
		let shell = globals
			.instantiate_exact::<wl_shell::WlShell, _>(1, |shell| shell.implement_dummy())
			.unwrap();
		let shell_surface = shell.get_shell_surface(&surface, |shell_surface|
			shell_surface.implement_closure(|event, shell_surface| {
				use wayland_client::protocol::wl_shell_surface::{Event};
				// This ping/pong mechanism is used by the wayland server to detect
				// unresponsive applications
				if let Event::Ping { serial } = event {
					shell_surface.pong(serial);
				}
			}, ())).unwrap();

		let mut ph = DrmPrimeHandle::default();
		ph.handle = handle;
		unsafe { drm_ioctl_prime_handle_to_fd(fd, &mut ph) }.unwrap();
		let cbfd = ph.fd;

		let linux_dmabuf = globals
			.instantiate_exact::<zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1, _>(1, |linux_dmabuf|
				linux_dmabuf.implement_closure(|event, linux_dmabuf| {
					use wayland_protocols::unstable::linux_dmabuf::v1::client::zwp_linux_dmabuf_v1::Event;
					match event {
						Event::Format {format} => 
							println!("found format {}{}{}{}",
								(format as u8 as char), ((format >> 8) as u8 as char),
								((format >> 16) as u8 as char), ((format >> 24) as u8 as char)),
						Event::Modifier {..} =>
							println!("modifier (ignored)"),
						_ => todo!()
					}
				}, ()))
			.unwrap();

		let window = HelloRadeonWindow {
			s: surface.clone(),
			buf: None, //buffer,
		};

		let buffer_params = linux_dmabuf.create_params(|buffer_params|
			buffer_params.implement(window, ())).expect("got no buffer params object");

		buffer_params.add(cbfd, 0, 0, W*4, 0, 0);

		shell_surface.set_toplevel();
		//surface.attach(Some(&buffer), 0, 0);
		surface.commit();

		(dmabufid, shell_surface, buffer_params)
	};

	let RGBX8888 = ('X' as u32) + (('R' as u32)<<8) + (('2' as u32)<<16) + (('4' as u32)<<24);
	buffer_params.create(W as i32, H as i32, RGBX8888, 0);

	loop {
		display.flush().unwrap();
		event_queue.dispatch().unwrap();
	}
}
