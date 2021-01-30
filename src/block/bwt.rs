pub(crate) fn inverse_bwt(tt: &mut [u32], orig_ptr: usize, c: &mut [u32; 256]) -> u32 {
    let mut sum = 0u32;

    for ci in c.iter_mut() {
        sum += *ci;
        *ci = sum - *ci;
    }

    for i in 0..tt.len() {
        let b = tt[i] & 0xff;
        tt[c[b as usize] as usize] |= (i as u32) << 8;
        c[b as usize] += 1;
    }

    tt[orig_ptr] >> 8
}
