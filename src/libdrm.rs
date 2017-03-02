
const DRM_DISPLAY_MODE_LEN: usize = 32;

#[repr(C)]
#[derive(Default)]
pub struct DrmModeCrtc {
	// set_connectors_ptr: u64,
	// count_connectors: u32,

	pub crtc_id: u32, /* Id */
	//fb_id: u32, /* Id of framebuffer */
	pub buffer_id: u32,

	pub x: u32, /* x Position on the framebuffer */
	pub y: u32, /* y Position on the framebuffer */

	pub width: u32,
	pub height: u32,

	//gamma_size: u32,
	pub mode_valid: u32,
	pub mode: DrmModeModeInfo,
	pub gamma_size: u32
}

#[repr(C)]
#[derive(PartialEq)]
pub enum DrmModeConnection {
	CONNECTED         = 1,
	DISCONNECTED      = 2,
	UNKNOWNCONNECTION = 3
}

#[repr(C)]
#[derive(PartialEq)]
pub enum DrmModeSubPixel {
	UNKNOWN        = 1,
	HORIZONTAL_RGB = 2,
	HORIZONTAL_BGR = 3,
	VERTICAL_RGB   = 4,
	VERTICAL_BGR   = 5,
	NONE           = 6
}

#[repr(C)]
// #[derive(Default)]
pub struct DrmModeConnector {
	pub connector_id: u32,
	pub encoder_id: u32, /* Encoder currently connected to */
	pub connector_type: u32,
	pub connector_type_id: u32,
	pub connection: DrmModeConnection,
	pub mmWidth: u32,
	pub mmHeight: u32, /* HxW in millimeters */
	pub subpixel: DrmModeSubPixel,

	pub count_modes: i32,
	pub modes: *const DrmModeModeInfo,

	pub count_props: i32,
	pub props: *const u32, /* List of property ids */
	pub prop_values: *const u64, /* List of property values */

	pub count_encoders: i32,
	pub encoders: *const u32 /* List of encoder ids */
}


#[repr(C)]
#[derive(Default)]
pub struct DrmModeEncoder {
	pub encoder_id: u32,
	pub encoder_type: u32,
	pub crtc_id: u32,
	pub possible_crtcs: u32,
	pub possible_clones: u32
}

#[repr(C)]
#[derive(Default)]
pub struct DrmModeModeInfo {
	pub clock: u32,
	pub hdisplay: u16,
	pub hsync_start: u16,
	pub hsync_end: u16,
	pub htotal: u16,
	pub hskew: u16,
	pub vdisplay: u16,
	pub vsync_start: u16,
	pub vsync_end: u16,
	pub vtotal: u16,
	pub vscan: u16,

	pub refresh: u32,

	pub flags: u32,
	pub ty/*pe*/: u32,
	pub name: [u8; DRM_DISPLAY_MODE_LEN]
}

#[repr(C)]
//#[derive(Default)]
pub struct DrmModeRes {
	pub count_fbs: i32,
	pub fbs: *const u32,

	pub count_crtcs: i32,
	pub crtcs: *const u32,

	pub count_connectors: i32,
	pub connectors: *const u32,

	pub count_encoders: i32,
	pub encoders: *const u32,

	pub min_width: u32,
	pub max_width: u32,
	pub min_height: u32,
	pub max_height: u32
}


#[link(name="drm")]
extern {
	pub fn drmModeGetCrtc(fd: i32, crtcId: u32) -> *const DrmModeCrtc;
	pub fn drmModeSetCrtc(fd: i32, crtcId: u32, bufferId: u32,
	                      x: u32, y: u32, connectors: *const u32, count: i32,
	                      mode: *const DrmModeModeInfo) -> i32;
	pub fn drmModeFreeCrtc(mode: *const DrmModeCrtc);
	pub fn drmModeGetResources(fd: i32) -> *const DrmModeRes;
	pub fn drmModeGetEncoder(fd: i32, encoder_id: u32)
		-> *const DrmModeEncoder;
	pub fn drmModeGetConnector(fd: i32, connector_id: u32)
		-> *const DrmModeConnector;
	pub fn drmModeFreeConnector(connector: *const DrmModeConnector);
	pub fn drmModeFreeEncoder(encoder: *const DrmModeEncoder);
	pub fn drmModeAddFB(fd: i32, width: u32, height: u32, depth: u8,
			bpp: u8, pitch: u32, bo_handle: u32,
			buf_id: *const u32) -> i32;
}
