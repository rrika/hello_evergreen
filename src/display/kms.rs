use std;
use libdrm::*;
use std::io::Error;
use super::*;

fn kms_select_connector(fd: i32, connector_ids: &[u32])
	-> Option<*const DrmModeConnector>
{
	for connector_id in connector_ids {
		let connector = unsafe {
			drmModeGetConnector(fd, *connector_id).as_ref()
		}.expect("got a null connector pointer");

		println!("connector {} found", connector.connector_id);
		if connector.connection == DrmModeConnection::CONNECTED
			&& connector.count_modes > 0
		{
			return Some(connector);
		}
		unsafe { drmModeFreeConnector(connector) };
	}
	None
}

fn kms_select_encoder(fd: i32, encoder_ids: &[u32],
                      connector: &DrmModeConnector)
	-> Option<*const DrmModeEncoder>
{
	for &encoder_id in encoder_ids {
		let encoder = unsafe {
			drmModeGetEncoder(fd, encoder_id).as_ref()
		}.expect("got a null encoder pointer");

		println!("encoder {} found", encoder.encoder_id);
		if encoder.encoder_id == connector.encoder_id
		{
			return Some(encoder);
		}
		unsafe { drmModeFreeEncoder(encoder) };
	}
	None
}

pub fn kmsmain(fd: i32, handle: u32) -> bool {

	unsafe {

	let resources = unsafe { drmModeGetResources(fd).as_ref() }
		.expect("libdrm gave no resources");

	let connector_ids = unsafe { std::slice::from_raw_parts(
		resources.connectors,
		resources.count_connectors as usize) };

	let encoder_ids = unsafe { std::slice::from_raw_parts(
		resources.encoders,
		resources.count_encoders as usize) };

	let connector = kms_select_connector(fd, connector_ids)
		.expect("no active connector found");

	let mode = unsafe { (*connector).modes.as_ref() }.unwrap();
	println!("({}x{})", mode.hdisplay, mode.vdisplay);

	let encoder = kms_select_encoder(fd, encoder_ids, &*connector)
		.expect("no matching encoder with connector, shouldn't happen");

	let mut pitch: u32 = W * 4; // (mode.hdisplay * 4) as u32;

	/* add FB which is associated with bo */
	let mut fb_id: u32 = 0;
	let ret = drmModeAddFB(fd,
		mode.hdisplay as u32,
		mode.vdisplay as u32,
		24, 32, pitch, handle, &mut fb_id);
	if ret != 0 {
		panic!("drmModeAddFB failed ({}x{}): {:?}\n",
			mode.hdisplay, mode.vdisplay, Error::last_os_error());
	}

	let connector_id: u32 = (*connector).connector_id;
	let crtc_id: u32 = (*encoder).crtc_id;

	unsafe {
		let backup: &DrmModeCrtc = drmModeGetCrtc(fd, crtc_id)
			.as_ref().unwrap();
		println!("set crtc 1");
		let ret1 = drmModeSetCrtc(fd, crtc_id, fb_id, 0, 0, &connector_id, 1, mode);
		::std::thread::sleep(::std::time::Duration::new(3, 0));
		println!("set crtc 2");
		let ret2 = drmModeSetCrtc(fd, backup.crtc_id,
			                          backup.buffer_id,
			                          backup.x,
			                          backup.y, &connector_id, 1,
			                          &backup.mode);
		drmModeFreeCrtc(backup);
	}
	true
	}
}
