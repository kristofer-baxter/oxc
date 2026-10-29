use std::{
    arch::aarch64::*,
    mem::{MaybeUninit, offset_of},
    sync::OnceLock,
};

use oxc_span::Span;

use crate::{
    pipeline::{cpu::Neon, lanes::Lanes},
    token::{TRIVIA_MAX, TRIVIA_MIN, is_trivia_byte, matches_tk, tk},
};

use super::common::{emit_value, invalid_diags};

static PAIR_LUTS: OnceLock<Box<PairLuts>> = OnceLock::new();

#[repr(C, align(128))]
pub struct PairLuts {
    lut0z: [[u8; 8]; 256],
    lutpad: [[u8; 32]; 256],
}

const _: () = assert!(offset_of!(PairLuts, lutpad).is_multiple_of(64));

impl PairLuts {
    #[target_feature(enable = "neon")]
    fn new(_cpu: Neon) -> Box<Self> {
        let mut luts = Box::<PairLuts>::new_uninit();
        let (lut0z, lutpad) = unsafe {
            unsafe fn uninit_array_mut<'a, const N: usize, T>(
                ptr: *mut [T; N],
            ) -> &'a mut [MaybeUninit<T>; N] {
                unsafe { ptr.cast::<[MaybeUninit<T>; N]>().as_mut().unwrap_unchecked() }
            }
            let ptr = luts.as_mut_ptr();
            (uninit_array_mut(&raw mut (*ptr).lut0z), uninit_array_mut(&raw mut (*ptr).lutpad))
        };
        for m in 0..256usize {
            let lut0z_line = lut0z[m].write([0; 8]);
            #[rustfmt::skip]
            let lutpad_line = lutpad[m].write([
                0,    0,    0,    0,    0,    0,    0,    0,
                0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
                0,    0,    0,    0,    0,    0,    0,    0,
                0,    0,    0,    0,    0,    0,    0,    0,
            ]);
            let mut k = 0usize;
            for bit in 0..8usize {
                if (m >> bit) & 1 != 0 {
                    lut0z_line[k] = bit as u8;
                    lutpad_line[k + 8] = (bit + 8) as u8;
                    k += 1;
                }
            }
        }
        unsafe { luts.assume_init() }
    }
}

pub(super) fn init_pair_luts_neon(cpu: Neon) {
    PAIR_LUTS.get_or_init(|| unsafe { PairLuts::new(cpu) });
}

#[inline(always)]
unsafe fn get_pair_luts() -> &'static PairLuts {
    unsafe { PAIR_LUTS.get().unwrap_unchecked() }
}

static SPAN_COMPACT: [[u8; 32]; 16] = span_compact();
static BCOMPACT8: [[u8; 8]; 256] = bcompact8();

const fn span_compact() -> [[u8; 32]; 16] {
    let mut t = [[0u8; 32]; 16];
    let mut mask = 0usize;
    while mask < 16 {
        let (mut q, mut k) = (0usize, 0usize);
        while q < 4 {
            if mask & (1 << q) != 0 {
                let mut byte = 0usize;
                while byte < 8 {
                    t[mask][8 * k + byte] = (8 * q + byte) as u8;
                    byte += 1;
                }
                k += 1;
            }
            q += 1;
        }
        mask += 1;
    }
    t
}

const fn bcompact8() -> [[u8; 8]; 256] {
    let mut t = [[0x80u8; 8]; 256];
    let mut mask = 0usize;
    while mask < 256 {
        let (mut b, mut k) = (0usize, 0usize);
        while b < 8 {
            if mask & (1 << b) != 0 {
                t[mask][k] = b as u8;
                k += 1;
            }
            b += 1;
        }
        mask += 1;
    }
    t
}

#[inline(never)]
#[target_feature(enable = "neon")]
pub(super) unsafe fn compress_blocks_neon(
    st: *const u64,
    kind: *const u8,
    b0: usize,
    b1: usize,
    starts: *mut u32,
    kinds: *mut u8,
    _cpu: Neon,
) -> usize {
    let pair_luts = get_pair_luts();
    let lut0z = pair_luts.lut0z.as_ptr().cast::<u8>();
    let lutpad = pair_luts.lutpad.as_ptr().cast::<u8>();

    let step16 = vdupq_n_u32(16);
    let zero8 = vdup_n_u8(0);
    let mut m = 0usize;
    for b in b0..b1 {
        let mword = *st.add(b);
        if mword == 0 {
            continue;
        }
        let base = b * 64;
        let o0: u32 = 0;
        let o1 = o0 + (mword & 0xffff).count_ones();
        let o2 = o1 + ((mword >> 16) & 0xffff).count_ones();
        let o3 = o2 + ((mword >> 32) & 0xffff).count_ones();
        let o4 = o3 + (mword >> 48).count_ones();
        let mut basev = vdupq_n_u32(base as u32);
        macro_rules! pair {
            ($p:expr, $off:expr) => {{
                let sub0 = ((mword >> (16 * $p)) & 0xff) as usize;
                let sub1 = ((mword >> (16 * $p + 8)) & 0xff) as usize;
                let pc0 = sub0.count_ones() as usize;
                let row0 = vcombine_u8(vld1_u8(lut0z.add(sub0 * 8)), zero8);
                let row1 = vld1q_u8(lutpad.add(sub1 * 32 + 8 - pc0));
                let ctrl = vorrq_u8(row0, row1);
                let kw = vld1q_u8(kind.add(base + 16 * $p));
                let ctrl_lo = vmovl_u8(vget_low_u8(ctrl));
                let ctrl_hi = vmovl_high_u8(ctrl);
                let dst = starts.add(m + $off as usize);
                vst1q_u32(dst, vaddq_u32(basev, vmovl_u16(vget_low_u16(ctrl_lo))));
                vst1q_u32(dst.add(4), vaddq_u32(basev, vmovl_high_u16(ctrl_lo)));
                vst1q_u32(dst.add(8), vaddq_u32(basev, vmovl_u16(vget_low_u16(ctrl_hi))));
                vst1q_u32(dst.add(12), vaddq_u32(basev, vmovl_high_u16(ctrl_hi)));
                basev = vaddq_u32(basev, step16);
                vqtbl1q_u8(kw, ctrl)
            }};
        }
        let ka = pair!(0, o0);
        let kb = pair!(1, o1);
        vst1q_u8(kinds.add(m + o0 as usize), ka);
        vst1q_u8(kinds.add(m + o1 as usize), kb);
        let kc = pair!(2, o2);
        let kd = pair!(3, o3);
        vst1q_u8(kinds.add(m + o2 as usize), kc);
        vst1q_u8(kinds.add(m + o3 as usize), kd);
        m += o4 as usize;
    }
    m
}

#[target_feature(enable = "neon")]
pub(super) unsafe fn build_spans_neon(
    stage_kind: *const u8,
    stage_pos: *const u32,
    m: usize,
    spans: *mut Span,
    sig_kinds: *mut u8,
    _cpu: Neon,
) -> usize {
    const BITS8: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];
    let sp = spans.cast::<u64>();
    let mut w = 0usize;
    let mut j = 0usize;

    let v_min = vdup_n_u8(TRIVIA_MIN);
    let v_span = vdup_n_u8(TRIVIA_MAX - TRIVIA_MIN);
    let v_hb = vdup_n_u8(tk!(Hashbang));
    let bits8 = vld1_u8(BITS8.as_ptr());

    while j + 8 <= m {
        let k8 = vld1_u8(stage_kind.add(j));
        let triv = vcle_u8(vsub_u8(k8, v_min), v_span);
        let drop = vbic_u8(triv, vceq_u8(k8, v_hb));
        let sig = (!vaddv_u8(vand_u8(drop, bits8))) as usize;

        let p0 = vld1q_u32(stage_pos.add(j));
        let p1 = vld1q_u32(stage_pos.add(j + 1));
        let tbl_a = uint8x16x2_t(
            vreinterpretq_u8_u32(vzip1q_u32(p0, p1)),
            vreinterpretq_u8_u32(vzip2q_u32(p0, p1)),
        );
        let ma = sig & 0xf;
        let ca_idx_ptr = SPAN_COMPACT[ma].as_ptr();
        vst1q_u8(sp.add(w).cast(), vqtbl2q_u8(tbl_a, vld1q_u8(ca_idx_ptr)));
        vst1q_u8(sp.add(w + 2).cast(), vqtbl2q_u8(tbl_a, vld1q_u8(ca_idx_ptr.add(16))));

        let p2 = vld1q_u32(stage_pos.add(j + 4));
        let p3 = vld1q_u32(stage_pos.add(j + 5));
        let tbl_b = uint8x16x2_t(
            vreinterpretq_u8_u32(vzip1q_u32(p2, p3)),
            vreinterpretq_u8_u32(vzip2q_u32(p2, p3)),
        );
        let mb = sig >> 4;
        let cb_idx_ptr = SPAN_COMPACT[mb].as_ptr();
        let wb = w + ma.count_ones() as usize;
        vst1q_u8(sp.add(wb).cast(), vqtbl2q_u8(tbl_b, vld1q_u8(cb_idx_ptr)));
        vst1q_u8(sp.add(wb + 2).cast(), vqtbl2q_u8(tbl_b, vld1q_u8(cb_idx_ptr.add(16))));

        let kc = vtbl1_u8(k8, vld1_u8(BCOMPACT8[sig].as_ptr()));
        vst1_u8(sig_kinds.add(w), kc);
        w += sig.count_ones() as usize;
        j += 8;
    }

    while j < m {
        let k = *stage_kind.add(j);
        *sp.add(w) = stage_pos.add(j).cast::<u64>().read_unaligned();
        *sig_kinds.add(w) = k;
        w += usize::from(!is_trivia_byte(k) || k == tk!(Hashbang));
        j += 1;
    }

    w
}

#[inline(always)]
unsafe fn mm64(a: uint8x16_t, b: uint8x16_t, c: uint8x16_t, d: uint8x16_t) -> u64 {
    const BITS: [u8; 16] = [1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128];
    let m = vld1q_u8(BITS.as_ptr());
    let ab = vpaddq_u8(vandq_u8(a, m), vandq_u8(b, m));
    let cd = vpaddq_u8(vandq_u8(c, m), vandq_u8(d, m));
    let s = vpaddq_u8(ab, cd);
    vgetq_lane_u64::<0>(vreinterpretq_u64_u8(vpaddq_u8(s, s)))
}

#[target_feature(enable = "neon")]
pub(super) unsafe fn lanes_post_neon(
    src: &[u8],
    out_kinds: *const u8,
    out_spans: *const Span,
    m: usize,
    nn: u32,
    lanes: &mut Lanes,
    _cpu: Neon,
) {
    let v_num = vdupq_n_u8(tk!(Number));
    let v_big = vdupq_n_u8(tk!(BigInt));
    let v_esc = vdupq_n_u8(tk!(IdentEscaped));
    let v_pesc = vdupq_n_u8(tk!(PrivateIdentEscaped));
    let v_inv = vdupq_n_u8(tk!(Invalid));

    macro_rules! hits {
        ($v:expr) => {{
            let v = $v;
            vorrq_u8(
                vorrq_u8(vceqq_u8(v, v_num), vceqq_u8(v, v_big)),
                vorrq_u8(vceqq_u8(v, v_esc), vceqq_u8(v, v_pesc)),
            )
        }};
    }

    let mut inv_acc = vdupq_n_u8(0);
    let mut i = 0usize;
    while i + 64 <= m {
        let v0 = vld1q_u8(out_kinds.add(i));
        let v1 = vld1q_u8(out_kinds.add(i + 16));
        let v2 = vld1q_u8(out_kinds.add(i + 32));
        let v3 = vld1q_u8(out_kinds.add(i + 48));

        let h0 = hits!(v0);
        let h1 = hits!(v1);
        let h2 = hits!(v2);
        let h3 = hits!(v3);

        let inv01 = vorrq_u8(vceqq_u8(v0, v_inv), vceqq_u8(v1, v_inv));
        let inv23 = vorrq_u8(vceqq_u8(v2, v_inv), vceqq_u8(v3, v_inv));
        inv_acc = vorrq_u8(inv_acc, vorrq_u8(inv01, inv23));

        let mut mask = mm64(h0, h1, h2, h3);
        while mask != 0 {
            emit_value(src, out_kinds, out_spans, i + mask.trailing_zeros() as usize, lanes);
            mask &= mask - 1;
        }
        i += 64;
    }

    let inv_u64 = vreinterpretq_u64_u8(inv_acc);
    let mut inv_dirty = (vgetq_lane_u64::<0>(inv_u64) | vgetq_lane_u64::<1>(inv_u64)) != 0;
    while i < m {
        let k = *out_kinds.add(i);
        if matches_tk!(k, Number | BigInt | IdentEscaped | PrivateIdentEscaped) {
            emit_value(src, out_kinds, out_spans, i, lanes);
        }
        inv_dirty |= k == tk!(Invalid);
        i += 1;
    }
    if inv_dirty {
        invalid_diags(src, out_kinds, out_spans, m, nn, lanes);
    }
}
