// using the ioctls directly was just done out of curiousity
// using libdrm is probably more portable

use std;
use std::marker::PhantomData;
use ioctl::*;

pub const RADEON_GEM_DOMAIN_CPU: u32 = 0x1;
pub const RADEON_GEM_DOMAIN_GTT: u32 = 0x2;
pub const RADEON_GEM_DOMAIN_VRAM: u32 = 0x4;

const DRM_IOCTL_BASE: u32 = b'd' as u32; // 0x64

const DRM_COMMAND_BASE: u32 = b'@' as u32; // 0x40;
const DRM_RADEON_GEM_INFO: u32			= 0x1c;
const DRM_RADEON_GEM_CREATE: u32		= 0x1d;
const DRM_RADEON_GEM_MMAP: u32			= 0x1e;
const DRM_RADEON_GEM_PREAD: u32			= 0x21;
const DRM_RADEON_GEM_PWRITE: u32		= 0x22;
const DRM_RADEON_GEM_SET_DOMAIN: u32	= 0x23;
const DRM_RADEON_GEM_WAIT_IDLE: u32		= 0x24;
const DRM_RADEON_CS: u32				= 0x26;
const DRM_RADEON_INFO: u32				= 0x27;
const DRM_RADEON_GEM_SET_TILING: u32	= 0x28;
const DRM_RADEON_GEM_GET_TILING: u32	= 0x29;
const DRM_RADEON_GEM_BUSY: u32			= 0x2a;
const DRM_RADEON_GEM_VA: u32			= 0x2b;
const DRM_RADEON_GEM_OP: u32			= 0x2c;
const DRM_RADEON_UCODE_UPDATE: u32		= 0x2e;

ioctl!(readwrite drm_ioctl_radeon_cs with DRM_IOCTL_BASE, DRM_COMMAND_BASE + DRM_RADEON_CS; DrmRadeonCs);
ioctl!(readwrite drm_ioctl_radeon_info with DRM_IOCTL_BASE, DRM_COMMAND_BASE + DRM_RADEON_INFO; DrmRadeonInfo);
ioctl!(write drm_ioctl_gem_close with DRM_IOCTL_BASE, 0x09; /*struct*/ DrmGemClose);
ioctl!(readwrite drm_ioctl_radeon_gem_busy with DRM_IOCTL_BASE, DRM_COMMAND_BASE + DRM_RADEON_GEM_BUSY; /*struct*/ DrmRadeonGemBusy);
ioctl!(readwrite drm_ioctl_radeon_gem_create with DRM_IOCTL_BASE, DRM_COMMAND_BASE + DRM_RADEON_GEM_CREATE; /*struct*/ DrmRadeonGemCreate);
ioctl!(readwrite drm_ioctl_radeon_gem_get_tiling with DRM_IOCTL_BASE, DRM_COMMAND_BASE + DRM_RADEON_GEM_GET_TILING; /*struct*/ DrmRadeonGemTiling);
ioctl!(readwrite drm_ioctl_radeon_gem_info with DRM_IOCTL_BASE, DRM_COMMAND_BASE + DRM_RADEON_GEM_INFO; /*struct*/ DrmRadeonGemInfo);
ioctl!(readwrite drm_ioctl_radeon_gem_mmap with DRM_IOCTL_BASE, DRM_COMMAND_BASE + DRM_RADEON_GEM_MMAP; /*struct*/ DrmRadeonGemMmap);
ioctl!(readwrite drm_ioctl_radeon_gem_op with DRM_IOCTL_BASE, DRM_COMMAND_BASE + DRM_RADEON_GEM_OP; /*struct*/ DrmRadeonGemOp);
ioctl!(readwrite drm_ioctl_radeon_gem_pread with DRM_IOCTL_BASE, DRM_COMMAND_BASE + DRM_RADEON_GEM_PREAD; /*struct*/ DrmRadeonGemPread);
ioctl!(readwrite drm_ioctl_radeon_gem_pwrite with DRM_IOCTL_BASE, DRM_COMMAND_BASE + DRM_RADEON_GEM_PWRITE; /*struct*/ DrmRadeonGemPwrite);
ioctl!(readwrite drm_ioctl_radeon_gem_set_domain with DRM_IOCTL_BASE, DRM_COMMAND_BASE + DRM_RADEON_GEM_SET_DOMAIN; /*struct*/ DrmRadeonGemSetDomain);
ioctl!(readwrite drm_ioctl_radeon_gem_set_tiling with DRM_IOCTL_BASE, DRM_COMMAND_BASE + DRM_RADEON_GEM_SET_TILING; /*struct*/ DrmRadeonGemTiling);
ioctl!(readwrite drm_ioctl_radeon_gem_va with DRM_IOCTL_BASE, DRM_COMMAND_BASE + DRM_RADEON_GEM_VA; /*struct*/ DrmRadeonGemVa);
ioctl!(write drm_ioctl_radeon_gem_wait_idle with DRM_IOCTL_BASE, DRM_COMMAND_BASE + DRM_RADEON_GEM_WAIT_IDLE; /*struct*/ DrmRadeonGemWaitIdle);
ioctl!(readwrite drm_ioctl_prime_handle_to_fd with DRM_IOCTL_BASE,
	/* how did this ever work */
	/* DRM_COMMAND_BASE + */
	0x2d; /*struct*/ DrmPrimeHandle);
ioctl!(write drm_ioctl_radeon_ucode_update with DRM_IOCTL_BASE, DRM_COMMAND_BASE + DRM_RADEON_UCODE_UPDATE; /*struct*/ DrmRadeonUcodeUpdate);

#[repr(C)]
#[derive(Default)]
pub struct DrmPrimeHandle {
	pub handle: u32,
	pub flags: u32,
	pub fd: i32
}

pub struct U64PtrSlice<'a, T: 'a> {
	ptrs: Vec<u64>,
	phantom: PhantomData<&'a T>
}

impl<'a, T> U64PtrSlice<'a, T> {
	pub fn new(items: &'a [T]) -> Self {
		let ptrs = items.iter().map(|r| r as *const T as u64).collect::<Vec<_>>();
		U64PtrSlice { ptrs: ptrs, phantom: PhantomData }
	}
	pub fn ptrs(&self) -> &[u64] { &self.ptrs[..] }
}

#[repr(C)]
#[derive(Default)]
pub struct DrmRadeonCsChunk {
	pub chunk_id: u32,
	pub length_dw: u32,
	pub chunk_data: u64,
}

/* DrmRadeonCsReloc.flags */

#[repr(C)]
#[derive(Default)]
pub struct DrmRadeonCsReloc {
	pub handle: u32,
	pub read_domains: u32,
	pub write_domain: u32,
	pub flags: u32,
}

#[repr(C)]
pub struct DrmRadeonCs<'a> {
	num_chunks: u32,
	pub cs_id: u32,
	/* this points to uint64_t * which point to cs chunks */
	chunks: u64,
	/* updates to the limits after this CS ioctl */
	pub gart_limit: u64,
	pub vram_limit: u64,
	phantom: PhantomData<&'a ()>
}

impl<'a> DrmRadeonCs<'a> {
	pub fn new(chunks: &U64PtrSlice<'a, DrmRadeonCsChunk>) -> Self {
		let chunk_ptrs = chunks.ptrs();
		DrmRadeonCs {
			num_chunks: (chunk_ptrs.len()) as u32,
			cs_id: 0,
			chunks: if chunk_ptrs.is_empty() {0} else {chunk_ptrs.as_ptr() as u64},
			gart_limit: 0,
			vram_limit: 0,
			phantom: PhantomData
		}
	}
}

#[repr(C)]
#[derive(Default)]
pub struct DrmGemClose {
	pub handle: u32,
	pub pad: u32,

}

#[repr(C)]
pub struct DrmRadeonInfo {
	pub request: u32,
	pub pad: u32,
	pub value: u64
}

#[repr(C)]
#[derive(Default)]
pub struct DrmRadeonGemBusy {
	pub handle: u32,
	pub domain: u32,

}

#[repr(C)]
#[derive(Default)]
pub struct DrmRadeonGemCreate {
	pub size: u64,
	pub alignment: u64,
	pub handle: u32,
	pub initial_domain: u32,
	pub flags: u32
}

#[repr(C)]
#[derive(Default)]
pub struct DrmRadeonGemTiling {
	pub handle: u32,
	pub tiling_flags: u32,
	pub pitch: u32,
}

#[repr(C)]
#[derive(Default)]
pub struct DrmRadeonGemInfo {
	pub gart_size: u64,
	pub vram_size: u64,
	pub vram_visible: u64,
}

#[repr(C)]
#[derive(Default)]
pub struct DrmRadeonGemMmap {
	pub handle: u32,
	pub pad: u32,
	pub offset: u64,
	pub size: u64,
	pub addr_ptr: u64,
}

#[repr(C)]
#[derive(Default)]
pub struct DrmRadeonGemOp {
	pub handle: u32, /* buffer */
	pub op: u32,     /* RADEON_GEM_OP_* */
	pub value: u64,  /* input or return value */
}

#[repr(C)]
#[derive(Default)]
pub struct DrmRadeonGemPread {
	/** Handle for the object being read. */
	pub handle: u32,
	pub pad: u32,
	/** Offset into the object to read from */
	pub offset: u64,
	/** Length of data to read */
	pub size: u64,
	/** Pointer to write the data into. */
	/* void *, but pointers are not 32/64 compatible */
	pub data_ptr: u64,
}

#[repr(C)]
#[derive(Default)]
pub struct DrmRadeonGemPwrite {
	/** Handle for the object being written to. */
	pub handle: u32,
	pub pad: u32,
	/** Offset into the object to write to */
	pub offset: u64,
	/** Length of data to write */
	pub size: u64,
	/** Pointer to read the data from. */
	/* void *, but pointers are not 32/64 compatible */
	pub data_ptr: u64,
}

#[repr(C)]
#[derive(Default)]
pub struct DrmRadeonGemSetDomain {
	pub handle: u32,
	pub read_domains: u32,
	pub write_domain: u32,
}

#[repr(C)]
#[derive(Default)]
pub struct DrmRadeonGemVa {
	pub handle: u32,
	pub operation: u32,
	pub vm_id: u32,
	pub flags: u32,
	pub offset: u64,
}

#[repr(C)]
#[derive(Default)]
pub struct DrmRadeonGemWaitIdle {
	pub handle: u32,
	pub pad: u32,
}

#[repr(C)]
#[derive(Default)]
pub struct DrmRadeonUcodeUpdate {
	pub component: u32,
	pub size: u32,
	pub data: u64,
}
