use std::collections::hash_map::HashMap;
use drm_radeon_ioctl::*;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Debug, FromPrimitive)]
pub enum Packet3 {
	NOP = 0x10,
	SET_BASE = 0x11,
	CLEAR_STATE = 0x12,
	INDEX_BUFFER_SIZE = 0x13,
	DISPATCH_DIRECT = 0x15,
	DISPATCH_INDIRECT = 0x16,
	INDIRECT_BUFFER_END = 0x17,
	MODE_CONTROL = 0x18,
	SET_PREDICATION = 0x20,
	REG_RMW = 0x21,
	COND_EXEC = 0x22,
	PRED_EXEC = 0x23,
	DRAW_INDIRECT = 0x24,
	DRAW_INDEX_INDIRECT = 0x25,
	INDEX_BASE = 0x26,
	DRAW_INDEX_2 = 0x27,
	CONTEXT_CONTROL = 0x28,
	DRAW_INDEX_OFFSET = 0x29,
	INDEX_TYPE = 0x2A,
	DRAW_INDEX = 0x2B,
	DRAW_INDEX_AUTO = 0x2D,
	DRAW_INDEX_IMMD = 0x2E,
	NUM_INSTANCES = 0x2F,
	DRAW_INDEX_MULTI_AUTO = 0x30,
	STRMOUT_BUFFER_UPDATE = 0x34,
	DRAW_INDEX_OFFSET_2 = 0x35,
	DRAW_INDEX_MULTI_ELEMENT = 0x36,
	MEM_SEMAPHORE = 0x39,
	MPEG_INDEX = 0x3A,
	COPY_DW = 0x3B,
	WAIT_REG_MEM = 0x3C,
	MEM_WRITE = 0x3D,
	INDIRECT_BUFFER = 0x32,
	CP_DMA = 0x41,
	PFP_SYNC_ME = 0x42,
	SURFACE_SYNC = 0x43,
	ME_INITIALIZE = 0x44,
	COND_WRITE = 0x45,
	EVENT_WRITE = 0x46,
	EVENT_WRITE_EOP = 0x47,
	EVENT_WRITE_EOS = 0x48,
	PREAMBLE_CNTL = 0x4A,
	RB_OFFSET = 0x4B,
	ALU_PS_CONST_BUFFER_COPY = 0x4C,
	ALU_VS_CONST_BUFFER_COPY = 0x4D,
	ALU_PS_CONST_UPDATE =         0x4E,
	ALU_VS_CONST_UPDATE =         0x4F,
	ONE_REG_WRITE = 0x57,
	SET_CONFIG_REG = 0x68,
	SET_CONTEXT_REG = 0x69,
	SET_ALU_CONST = 0x6A,	
	SET_BOOL_CONST = 0x6B,
	SET_LOOP_CONST = 0x6C,
	SET_RESOURCE = 0x6D,
	SET_SAMPLER = 0x6E,
	SET_CTL_CONST = 0x6F,
	SET_RESOURCE_OFFSET = 0x70,
	SET_ALU_CONST_VS = 0x71,
	SET_ALU_CONST_DI = 0x72,
	SET_CONTEXT_REG_INDIRECT = 0x73,
	SET_RESOURCE_INDIRECT = 0x74,
	SET_APPEND_CNT = 0x75,
	EVENT_WRITE_7E = 0x7E, // disregard this
}

const RADEON_CHUNK_ID_RELOCS: u32 = 0x01;
const RADEON_CHUNK_ID_IB: u32 = 0x02;
const RADEON_CHUNK_ID_FLAGS: u32 = 0x03;
// const RADEON_CHUNK_ID_CONST_IB: u32 = 0x04;

#[derive(Default)]
pub struct CS {
	ib: Vec<u32>,
	relocs: Vec<DrmRadeonCsReloc>,
	labels: HashMap<usize, String>
}

impl CS {
	pub fn emit(&mut self, data: u32) {
		self.ib.push(data);
	}
	pub fn write(&mut self, data: &[u32]) {
		self.ib.extend_from_slice(data);
	}
	pub fn write_label(&mut self, label: &str) {
		self.labels.insert(self.ib.len(), label.to_owned());
	}
	pub fn write_reloc(&mut self, handle: u32, read_domains: u32, write_domain: u32, flags: u32) {
		self.relocs.push(DrmRadeonCsReloc{
			handle: handle,
			read_domains: read_domains,
			write_domain: write_domain,
			flags: flags
		})
	}
	pub fn set_reg(&mut self, reg: u32, value: u32) {
		self.set_reg_n(reg, 1);
		self.emit(value);
	}
	pub fn set_reg_n(&mut self, reg: u32, num: u32) {
		if reg >= SET_CONFIG_REG__OFFSET && reg < SET_CONFIG_REG__END {
			self.emit(packet3(Packet3::SET_CONFIG_REG, num, 0));
			self.emit((reg - SET_CONFIG_REG__OFFSET) >> 2);
		} else if reg >= SET_CONTEXT_REG__OFFSET && reg < SET_CONTEXT_REG__END {
			self.emit(packet3(Packet3::SET_CONTEXT_REG, num, 0));
			self.emit((reg - SET_CONTEXT_REG__OFFSET) >> 2);
		} else if reg >= SET_RESOURCE__OFFSET && reg < SET_RESOURCE__END {
			self.emit(packet3(Packet3::SET_RESOURCE, num, 0));
			self.emit((reg - SET_RESOURCE__OFFSET) >> 2);
		} else if reg >= SET_SAMPLER__OFFSET && reg < SET_SAMPLER__END {
			self.emit(packet3(Packet3::SET_SAMPLER, num, 0));
			self.emit((reg - SET_SAMPLER__OFFSET) >> 2);
		} else if reg >= SET_CTL_CONST__OFFSET && reg < SET_CTL_CONST__END {
			self.emit(packet3(Packet3::SET_CTL_CONST, num, 0));
			self.emit((reg - SET_CTL_CONST__OFFSET) >> 2);
		} else if reg >= SET_LOOP_CONST__OFFSET && reg < SET_LOOP_CONST__END {
			self.emit(packet3(Packet3::SET_LOOP_CONST, num, 0));
			self.emit((reg - SET_LOOP_CONST__OFFSET) >> 2);
		} else if reg >= SET_BOOL_CONST__OFFSET && reg < SET_BOOL_CONST__END {
			self.emit(packet3(Packet3::SET_BOOL_CONST, num, 0));
			self.emit((reg - SET_BOOL_CONST__OFFSET) >> 2);
		} else {
			self.emit((num << 8) + (reg >> 2));
		};
	}
	pub fn submit(&self, fd: i32) {
		// Three chunks: instruction buffer, relocations and flags.
		let flags: [u32; 2] = [0, 0];
		if true {
			for (i, word) in self.ib.iter().enumerate() {
				if let Some(label) = self.labels.get(&i) {
					println!("  {}", label);
				}
				println!("[{:2}] = {:08x}", i, word);
			}
		}
		let chunk0 = DrmRadeonCsChunk {
			chunk_id: RADEON_CHUNK_ID_IB,
			length_dw: self.ib.len() as u32,
			chunk_data: if self.ib.is_empty() {0} else {&self.ib[0] as *const u32 as u64}
		};
		let chunk1 = DrmRadeonCsChunk {
			chunk_id: RADEON_CHUNK_ID_RELOCS,
			length_dw: self.relocs.len() as u32 * 4,
			chunk_data: if self.relocs.is_empty() {0} else {&self.relocs[0] as *const DrmRadeonCsReloc as u64}
		};
		let chunk2 = DrmRadeonCsChunk {
			chunk_id: RADEON_CHUNK_ID_FLAGS,
			length_dw: 2,
			chunk_data: &flags[0] as *const u32 as u64
		};
		let chunks: [DrmRadeonCsChunk; 3] = [chunk0, chunk1, chunk2];
		// Finally, fill in the arguments for the ioctl.
		let ahh = U64PtrSlice::new(&chunks);
		let mut cs = DrmRadeonCs::new(&ahh);
		unsafe { drm_ioctl_radeon_cs(fd, &mut cs) };
	}
}

const SET_CONFIG_REG__OFFSET: u32  = 0x00008000;
const SET_CONFIG_REG__END: u32     = 0x0000ac00;
const SET_CONTEXT_REG__OFFSET: u32 = 0x00028000;
const SET_CONTEXT_REG__END: u32    = 0x00029000;
const SET_RESOURCE__OFFSET: u32    = 0x00030000;
const SET_RESOURCE__END: u32       = 0x00038000;
const SET_SAMPLER__OFFSET: u32     = 0x0003c000;
const SET_SAMPLER__END: u32        = 0x0003c600;
const SET_CTL_CONST__OFFSET: u32   = 0x0003cff0;
const SET_CTL_CONST__END: u32      = 0x0003ff0c;
const SET_LOOP_CONST__OFFSET: u32  = 0x0003a200;
const SET_LOOP_CONST__END: u32     = 0x0003a500;
const SET_BOOL_CONST__OFFSET: u32  = 0x0003a500;
const SET_BOOL_CONST__END: u32     = 0x0003a518;

pub struct ColorBuffer {
	pub base:        u32,
	pub pitch:       u32,
	pub slice:       u32,
	pub view:        u32,
	pub info:        u32,
	pub attrib:      u32,
	pub dim:         u32,
	// pub cmask:       u32,
	// pub cmask_slice: u32,
	// pub fmask:       u32,
	// pub fmask_slice: u32,
	pub clear_word0: u32,
	pub clear_word1: u32,
	pub clear_word2: u32,
	pub clear_word3: u32
}

pub fn packet3(op: Packet3, n: u32, c: u32) -> u32 {
	(/*RADEON_PACKET_TYPE*/3 << 30) | ((op as u32 & 0xFF) << 8) | ((n & 0x3FFF) << 16) | (if c != 0 {1} else {0})
}
pub const CACHE_FLUSH_AND_INV_EVENT_TS: u32 = (0x14 << 0);
pub const CACHE_FLUSH_AND_INV_EVENT: u32 = (0x16 << 0);
const VGT_INDEX_16: u32 = 0;
const VGT_INDEX_32: u32 = 1;
const VGT_DMA_SWAP_NONE: u32 = (0 << 2);
const VGT_DMA_SWAP_16_BIT: u32 = (1 << 2);
const VGT_DMA_SWAP_32_BIT: u32 = (2 << 2);
const VGT_DMA_SWAP_WORD: u32 = (3 << 2);
const V_0287F0_DI_SRC_SEL_IMMEDIATE: u32 = 1;
const R_008958_VGT_PRIMITIVE_TYPE: u32 = 0x8958;
const V_008958_DI_PT_TRILIST: u32 = 0x0004;
pub fn event_index(n: u32) -> u32 { n << 8 }
pub fn data_sel(n: u32) -> u32 { n << 29 }
pub fn int_sel(n: u32) -> u32 { n << 24 }

const NUM_GPRS_shift: u32 = 0;
const STACK_SIZE_shift: u32 = 8;

pub struct ShaderConfig {
	pub shader_addr: u32,
	pub shader_size: u32,
	pub num_gprs: u32,
	pub stack_size: u32,
	//bo: u32
	pub export_mode: u32, // for PS
	pub clamp_consts: u32 // for PS
}

pub struct VtxRes {
	pub byteoffset: u32,
	pub bytesize: u32,
	pub stride: u32,
	pub vtxcount: u32,
}
pub struct DrawInfo<'a> {
	pub indirect: bool,
	pub indexed: bool,
	pub index_size: u32,
	pub user_buffer: Option<&'a [u32]>,
	pub instance_count: u32
}

pub fn setup_depth(cs: &mut CS) {
	cs.set_reg(0x28800, 0); // DB_DEPTH_CONTROL // disable stencil and depth
	// cs.set_reg(0x28ac0, 7); // DB_SRESULTS_COMPARE_STATE0 // always pass
	// cs.set_reg(0x28780, 0x40000001); // CB_BLEND0_CONTROL // RT0: enable and dst' = src * 1 + dst * 0
}

pub fn setup_scissors(cs: &mut CS, w: u32, h: u32) {
	cs.set_reg_n(0x28240, 2); // PA_SC_GENERIC_SCISSOR_TL
	cs.emit(0);
	cs.emit(w + (h << 14));
	cs.set_reg_n(0x28030, 2); // PA_SC_SCREEN_SCISSOR_TL
	cs.emit(0);
	cs.emit(w + (h << 14));
	cs.set_reg_n(0x28204, 2); // PA_SC_WINDOW_SCISSOR_TL
	cs.emit(0);
	cs.emit(w + (h << 14));

	cs.set_reg(0x28810, 0x00010000); // PA_CL_CLIP_CNTL
	cs.set_reg(0x28818, 0x00000100); // PA_CL_VTE_CNTL
}

pub fn setup_fb(cs: &mut CS, w: u32, h: u32, tiled: bool, bo_reloc: &Fn(&mut CS) -> ()) {

	let w8 = (w+7)/8;
	let h8 = (h+7)/8;

	let cb = ColorBuffer {
		base:  0, // offset_of!(BOLayout=>cb) as u32 >> 8,
		pitch: w8-1,
		slice: w8*h8-1,
		view: 0, /* allow slices 0 to 0 */
		info: (26/*RGBA*/<<2) + (6/*SRGB*/<<12) + if tiled {
			(4<<8) /* ARRAY_2D_TILED_THIN1 (8×8×1 macrotiles) */
		} else { 0 },
		attrib: if tiled {
			(3<<5)
		} else {
			16 /* NON_DISP_TILING_ORDER */
		},
		dim: ((h-1)<<16) + (w-1),
		clear_word0: 0,
		clear_word1: 0,
		clear_word2: 0,
		clear_word3: 0
	};


	cs.set_reg(0x28c60, cb.base);        // CB_COLOR0_BASE
	bo_reloc(cs); // CB_COLOR0_BASE
	cs.set_reg(0x28c7c, 0 /*cb.cmask*/); // CB_COLOR0_CMASK
	bo_reloc(cs); // CB_COLOR0_CMASK
	cs.set_reg(0x28c84, 0 /*cb.fmask*/); // CB_COLOR0_FMASK
	bo_reloc(cs); // CB_COLOR0_FMASK
	cs.set_reg(0x28c74, cb.attrib);      // CB_COLOR0_ATTRIB
	bo_reloc(cs); // CB_COLOR0_ATTRIB
	cs.set_reg(0x28c70, cb.info);        // CB_COLOR0_INFO
	bo_reloc(cs); // CB_COLOR0_INFO
	cs.set_reg(0x28c64, cb.pitch);       // CB_COLOR0_PITCH
	cs.set_reg(0x28c68, cb.slice);       // CB_COLOR0_SLICE
	cs.set_reg(0x28c6c, cb.view);        // CB_COLOR0_VIEW
	cs.set_reg(0x28c78, cb.dim);         // CB_COLOR0_DIM
	cs.set_reg(0x28c80, 0 /*cb.cmask_slice*/); // CB_COLOR0_CMASK_SLICE
	cs.set_reg(0x28c88, 0 /*cb.fmask_slice*/); // CB_COLOR0_FMASK_SLICE
	cs.set_reg_n(0x28c8c, 4);
	cs.emit(/*0x28c8c, */ cb.clear_word0); // CB_COLOR0_CLEAR_WORD0
	cs.emit(/*0x28c90, */ cb.clear_word1); // CB_COLOR0_CLEAR_WORD1
	cs.emit(/*0x28c94, */ cb.clear_word2); // CB_COLOR0_CLEAR_WORD2
	cs.emit(/*0x28c98, */ cb.clear_word3); // CB_COLOR0_CLEAR_WORD3
	cs.set_reg(0x28238, 15); // 0xCB_TARGET_MASK
	cs.set_reg(0x28808, 0x00cc0010); // CB_COLOR_CONTROL
	cs.set_reg(0x28780, 0); // CB_BLEND0_CONTROL
}

pub fn setup_spi<'a>(cs: &'a mut CS) {
	cs.write_label("setting up spi");
	if false { // already done above
		cs.set_reg(0x28644, 0x00000401); /* SPI_PS_INPUT_CNTL_0 */ /* 4=flat shader 1=semantic 1?? */
	} else {
		cs.set_reg(0x286C4, 0); /* SPI_VS_OUT_CONFIG */
	}
	
	cs.set_reg_n(0x286cc, 3);
	if true {
		cs.emit(/*0x286cc,*/ 0x20000000); /* SPI_PS_IN_CONTROL_0 */ /* do it like radeondemo */
	} else {
		cs.emit(/*0x286cc,*/ 0x10000001); /* SPI_PS_IN_CONTROL_0 */ /* no position but perspective gradients */
	}
	cs.emit(/*0x286d0,*/ 0x00000000); /* SPI_PS_IN_CONTROL_1 */
	cs.emit(/*0x286d4,*/ 0x00000000); /* SPI_INTERP_CONTROL_0 */


	if false {
		cs.set_reg(0x286e0, 0x00000100); /* SPI_BARYC_CNTL */ /* enable linear gradients */
		cs.set_reg(0x286d8, 0x00000000); /* SPI_INPUT_Z */ /* no comment */
	} else {
		cs.set_reg(0x286e0, 0x00100000); /* SPI_BARYC_CNTL */ /* enable perspective gradients */
	}

    cs.set_reg(0x286e4, 0x00000000); /* SPI_PS_IN_CONTROL_2 */
}

const TC_ACTION_ENA_bit: u32 = 1 << 23;
const SH_ACTION_ENA_bit: u32 = 1 << 27;

// surface_sync(cs, SH_ACTION_ENA_bit, , );
pub fn surface_sync(cs: &mut CS, sync_type: u32, cp_coher_size: u32, mc_addr: u32, number: u64, bo_reloc: &Fn(&mut CS) -> ()) {
	// write_number(cs, 0x10000+number);
	bo_reloc(cs);
	cs.write_label("surface sync");
	cs.write(&[
		packet3(Packet3::SURFACE_SYNC, 3, 0),
		sync_type,
		(cp_coher_size+255) >> 8,
		mc_addr >> 8,
		10 // poll interval
	]);
	bo_reloc(cs);
}

pub fn set_ps_const_buffer(cs: &mut CS, offset: u32, bo_reloc: &Fn(&mut CS) -> ()) {
	let size = 1;
	surface_sync(cs, SH_ACTION_ENA_bit, size, offset, 1, bo_reloc);
	cs.set_reg(0x28140, 1);         // SQ_ALU_CONST_BUFFER_SIZE_PS_0
	cs.set_reg(0x28940, offset>>8); // SQ_ALU_CONST_CACHE_PS_0
	bo_reloc(cs);
}

pub fn set_vtx_resource<'a>(cs: &'a mut CS, vtxres: &VtxRes, bo_reloc: &Fn(&mut CS) -> ()) {
	let base: u32 = 0x30000 + 8 * 4 * 176;
	surface_sync(cs, TC_ACTION_ENA_bit, vtxres.bytesize, vtxres.byteoffset, 2, bo_reloc);

	cs.write_label("setting up vtx resource");
	cs.set_reg_n(base, 8);
	cs.emit(vtxres.byteoffset);
	cs.emit(vtxres.bytesize-1);
	cs.emit((vtxres.stride<<8));
	cs.emit(0x3440); // w z y x cached
	cs.emit(vtxres.vtxcount);
	cs.emit(0);
	cs.emit(0);
	cs.emit(3<<30); // valid buffer
}

// ES = export shader
// FS = fetch shader
// GS = geometry shader
// HS = hull shader
// LS = vertex to lds
// PS = pixel shader
// VS = vertex shader

pub fn evergreen_vs_setup(cs: &mut CS, vs_conf: &ShaderConfig, bo_reloc: &Fn(&mut CS) -> ()) {
	cs.write_label("evergreen_vs_setup");

	let mut sq_pgm_resources: u32;
	let mut sq_pgm_resources_2: u32 = 0;

	sq_pgm_resources =  (vs_conf.num_gprs << NUM_GPRS_shift) |
						(vs_conf.stack_size << STACK_SIZE_shift);

	/* flush SQ cache */
	// evergreen_cp_set_surface_sync(radeon, SH_ACTION_ENA_bit,
	// 			  vs_conf.shader_size, vs_conf->shader_addr,
	// 			  vs_conf.bo, domain, 0);
	surface_sync(cs, SH_ACTION_ENA_bit, vs_conf.shader_size, vs_conf.shader_addr, 3, bo_reloc);

	cs.set_reg(0x2885c, vs_conf.shader_addr >> 8); // SQ_PGM_START_VS
	bo_reloc(cs);

	cs.set_reg_n(0x28860, 2); // SQ_PGM_RESOURCES_VS
	cs.emit(sq_pgm_resources);
	cs.emit(sq_pgm_resources_2);
}

pub fn evergreen_ps_setup(cs: &mut CS, ps_conf: &ShaderConfig, bo_reloc: &Fn(&mut CS) -> ()) {
	cs.write_label("evergreen_ps_setup");

	let sq_pgm_resources: u32;
	let sq_pgm_resources_2: u32 = 0;

	sq_pgm_resources = (ps_conf.num_gprs << NUM_GPRS_shift) |
					   (ps_conf.stack_size << STACK_SIZE_shift);

	/*if (ps_conf->dx10_clamp)
	sq_pgm_resources |= DX10_CLAMP_bit;
	if (ps_conf->uncached_first_inst)
	sq_pgm_resources |= UNCACHED_FIRST_INST_bit;
	if (ps_conf->clamp_consts)
	sq_pgm_resources |= CLAMP_CONSTS_bit;*/

	// sq_pgm_resources_2 = ((ps_conf->single_round << SINGLE_ROUND_shift) |
	// 		  (ps_conf->double_round << DOUBLE_ROUND_shift));

	/*if (ps_conf->allow_sdi)
	sq_pgm_resources_2 |= ALLOW_SINGLE_DENORM_IN_bit;
	if (ps_conf->allow_sd0)
	sq_pgm_resources_2 |= ALLOW_SINGLE_DENORM_OUT_bit;
	if (ps_conf->allow_ddi)
	sq_pgm_resources_2 |= ALLOW_DOUBLE_DENORM_IN_bit;
	if (ps_conf->allow_ddo)
	sq_pgm_resources_2 |= ALLOW_DOUBLE_DENORM_OUT_bit;*/

	/* flush SQ cache */
	// evergreen_cp_set_surface_sync(radeon, SH_ACTION_ENA_bit,
	// 			  ps_conf->shader_size, ps_conf->shader_addr,
	// 			  ps_conf->bo, domain, 0);
	surface_sync(cs, SH_ACTION_ENA_bit, ps_conf.shader_size, ps_conf.shader_addr, 4, bo_reloc);

	cs.set_reg(0x28840, ps_conf.shader_addr >> 8); // SQ_PGM_START_FS
	bo_reloc(cs);

	cs.set_reg_n(0x28844, 3); // SQ_PGM_RESOURCES_PS
	cs.emit(sq_pgm_resources);    // SQ_PGM_RESOURCES_PS
	cs.emit(sq_pgm_resources_2);  // SQ_PGM_RESOURCES_2_PS
	cs.emit(ps_conf.export_mode); // SQ_PGM_EXPORTS_PS
}


pub fn vbo<'a>(cs: &'a mut CS, info: DrawInfo) {
	// see r600_draw_vbo in mesa

	let render_cond_bit: u32 = 0;

	if true {
		let ls_hs_config: u32 = 0;
		let lds_alloc: u32 = 0;
		if false {
			cs.set_reg(/*R_*/0x028B58/*_VGT_LS_HS_CONFIG*/, ls_hs_config); // not needed?
			cs.set_reg(/*R_*/0x0288E8/*_SQ_LDS_ALLOC*/, lds_alloc);        // already done
		}
	}
	if false {
		// not used in radeondemo
		cs.set_reg(/*R_*/0x03CFF4/*_SQ_VTX_START_INST_LOC*/, 0);
	}
	const V_008958_DI_PT_TRILIST: u32 = 4;
	cs.set_reg(R_008958_VGT_PRIMITIVE_TYPE, V_008958_DI_PT_TRILIST);

	if info.indexed {
		let r600_big_endian = false;
		cs.emit(packet3(Packet3::INDEX_TYPE, 0, 0));
		cs.emit(if info.index_size == 4
				{VGT_INDEX_32 | (if r600_big_endian {VGT_DMA_SWAP_32_BIT} else {0})} else
				{VGT_INDEX_16 | (if r600_big_endian {VGT_DMA_SWAP_16_BIT} else {0})});
		if !info.indirect {
			cs.emit(packet3(Packet3::NUM_INSTANCES, 0, 0));
			cs.emit(info.instance_count);
		}
		if let Some(ub) = info.user_buffer {
			//let size_bytes: u32 = ub.len()*info.index_size;
			let size_dw: u32 = ub.len() as u32; //(size_bytes+3)/4;
			cs.write_label("DRAW_INDEX_IMMD");
			cs.emit(packet3(Packet3::DRAW_INDEX_IMMD, 1 + size_dw, render_cond_bit));
			cs.emit(ub.len() as u32);
			cs.emit(V_0287F0_DI_SRC_SEL_IMMEDIATE);
			cs.write(&ub);
		}
	}
}
