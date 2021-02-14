use std::mem;

pub(crate) fn inverse_bwt(tt: &mut [u32], orig_ptr: usize, mut c: [u32; 256]) -> u32 {
    let mut sum = 0u32;

    for ci in c.iter_mut() {
        sum += mem::replace(ci, sum);
    }

    for i in 0..tt.len() {
        let b = (tt[i] & 0xff) as usize;
        tt[c[b] as usize] |= (i as u32) << 8;
        c[b] += 1;
    }

    tt[orig_ptr] >> 8
}
