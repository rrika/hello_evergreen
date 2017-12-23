use xcb;
use super::*;
use drm_radeon_ioctl::*;

#[link(name="xshmfence")]
extern {
	fn xshmfence_alloc_shm() -> i32;
	fn xshmfence_map_shm(fence_fd: i32) -> usize;
	fn xshmfence_trigger(fence: usize);
	fn xshmfence_reset(fence: usize);
	fn xshmfence_unmap_shm(fence: usize);
}

pub fn xcbmain(fd: i32, handle: u32) -> bool {

	let (conn, screen_num) = if let Ok(z) = xcb::Connection::connect(None) { z } else { return false };
	let setup = conn.get_setup();
	let screen = setup.roots().nth(screen_num as usize).unwrap();

	let foreground = conn.generate_id();

	xcb::create_gc(&conn, foreground, screen.root(), &[
			(xcb::GC_FOREGROUND, screen.black_pixel()),
			(xcb::GC_GRAPHICS_EXPOSURES, 0),
	]);

	let win = conn.generate_id();
	xcb::create_window(&conn,
		xcb::COPY_FROM_PARENT as u8,
		win,
		screen.root(),
		0, 0,
		//W*2 + 60, H + 20,
		(W + 20) as u16, (H + 20) as u16,
		10,
		xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
		screen.root_visual(), &[
			(xcb::CW_BACK_PIXEL, screen.white_pixel()),
			(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_EXPOSURE),
		]
	);
	xcb::map_window(&conn, win);
	conn.flush();

	let (wm_protocols, wm_delete_window) = {
		let pc = xcb::intern_atom(&conn, false, "WM_PROTOCOLS");
		let dwc = xcb::intern_atom(&conn, false, "WM_DELETE_WINDOW");

		let p = match pc.get_reply() {
			Ok(p) => p.atom(),
			Err(_) => panic!("could not load WM_PROTOCOLS atom")
		};
		let dw = match dwc.get_reply() {
			Ok(dw) => dw.atom(),
			Err(_) => panic!("could not load WM_DELETE_WINDOW atom")
		};
		(p, dw)
	};

	let protocols = [wm_delete_window];
	xcb::change_property(&conn, xcb::PROP_MODE_REPLACE as u8,
			win, wm_protocols, xcb::ATOM_ATOM, 32, &protocols);

	let mut ph = DrmPrimeHandle::default();
	ph.handle = handle;
	unsafe { drm_ioctl_prime_handle_to_fd(fd, &mut ph) };
	let cbfd = ph.fd;
	// let dbfd = ;

	let cbpixmap = conn.generate_id();
	// let dbpixmap = conn.generate_id();
	xcb::dri3::pixmap_from_buffer_checked(&conn, cbpixmap, screen.root(), L_CB_SIZE as u32, W as u16, H as u16, (W*4) as u16, 24, 32, cbfd);
	conn.has_error().unwrap();
	// xcb::dri3::pixmap_from_buffer(&conn, dbpixmap, screen.root(), L_DB_SIZE, W, H, W, 1, 32, dbfd);

	let cbpixmap_sync_fence = conn.generate_id();

	let fence_fd = unsafe { xshmfence_alloc_shm() };
	if fence_fd < 0 { panic!("xshmfence_alloc_shm failed"); }
	let shm_fence = unsafe { xshmfence_map_shm(fence_fd) };

	xcb::dri3::fence_from_fd(&conn, cbpixmap, cbpixmap_sync_fence, false, fence_fd);
	conn.has_error().unwrap();

	unsafe { xshmfence_trigger(shm_fence); }

	conn.flush();

	loop {
		let event = conn.wait_for_event();
		match event {
			None => { break; }
			Some(event) => {
				let r = event.response_type() & !0x80;
				match r {
					xcb::EXPOSE => {
						unsafe { xshmfence_reset(shm_fence); }
						xcb::copy_area_checked(&conn, cbpixmap, win, foreground, 0, 0, 10, 10, W as u16, H as u16);
						unsafe { xshmfence_trigger(shm_fence); }
						/* We flush the request */
						conn.flush();
					},
					xcb::CLIENT_MESSAGE => {
						let cmev = unsafe { xcb::cast_event::<xcb::ClientMessageEvent>(&event) };
						if cmev.type_() == wm_protocols && cmev.format() == 32 {
							let protocol = cmev.data().data32()[0];
							if protocol == wm_delete_window {
								break;
							}
						}
					},
					_ => {}
				}
			}
		}
	}
	true
}
