pub mod kms;
pub mod wayland;
pub mod xcb;

pub use self::kms::kmsmain;
pub use self::wayland::waylandmain;
pub use self::xcb::xcbmain;

pub const TILED: bool = false;

pub const W: u32 = 1408;
pub const H: u32 = 768;
//pub const W: u32 = 640;
//pub const H: u32 = 480;
pub const L_CB_SIZE: usize = (W*H*4) as usize;
pub const L_DB_SIZE: usize = (W*H*4) as usize;
pub const L_SHADERBLOB_SIZE: usize = 4096;
pub const L_VERTEXBUFFER_SIZE: usize = 4*4*4;
pub const SH_SOLID_VS_OFFSET: usize = 0;
pub const SH_SOLID_PS_OFFSET: usize = 512;
// pub const SH_COPY_VS_OFFSET: usize = 1024;
// pub const SH_COPY_PS_OFFSET: usize = 1536;
// pub const SH_COMP_VS_OFFSET: usize = 2048;
// pub const SH_COMP_PS_OFFSET: usize = 2560;
// pub const SH_XV_VS_OFFSET: usize = 3072;
// pub const SH_XV_PS_OFFSET: usize = 3584;
