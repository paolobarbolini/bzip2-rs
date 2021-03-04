use std::cmp::{max, min};

const N_RADIX: usize = 2;
const N_QSORT: usize = 12;
const N_SHELL: usize = 18;
const N_OVERSHOOT: usize = N_RADIX + N_QSORT + N_SHELL + 2;

const BYTE_RANGE: usize = 256;
const TWO_BYTE_RANGE: usize = BYTE_RANGE * BYTE_RANGE;

fn fallback_simple_sort(fmap: &mut [u32], eclass: &[u32], lo: usize, hi: usize) {
    // original code had lo == hi; this bound covers more cases
    if lo >= hi {
        return;
    }

    fn do_sort(_fmap: &mut [u32], _eclass: &[u32], _lo: usize, _hi: usize, _jump: usize) {
        let mut i = (_hi - _jump) as isize;
        while i >= _lo as isize {
            let tmp = _fmap[i as usize] as usize;
            let ec_tmp = _eclass[tmp];

            let mut j = i as usize + _jump;
            while j <= _hi && ec_tmp > _eclass[_fmap[j] as usize] {
                _fmap[j - _jump] = _fmap[j];
                j += _jump;
            }

            i -= 1;
        }
    }

    if hi - lo > 3 {
        do_sort(fmap, eclass, lo, hi, 4);
    }

    do_sort(fmap, eclass, lo, hi, 1);
}

/// Swaps `n_elem` elements in `arr` starting from indices `i1` and `i2`
fn swap_regions(arr: &mut [u32], mut n_elem: usize, mut i1: usize, mut i2: usize) {
    while n_elem > 0 {
        arr.swap(i1, i2);
        i1 += 1;
        i2 += 1;
        n_elem -= 1;
    }
}

fn fallback_qsort_3(fmap: &mut [u32], eclass: &[u32], lo_st: usize, hi_st: usize) {
    const FALLBACK_QSORT_STACK_SIZE: usize = 100;
    const FALLBACK_QSORT_SMALL_THRESH: usize = 10;

    // stack pointer
    let mut r = 0;
    let mut stack = Vec::with_capacity(FALLBACK_QSORT_STACK_SIZE);

    stack.push((lo_st, hi_st));

    while !stack.is_empty() {
        assert!(stack.len() < FALLBACK_QSORT_STACK_SIZE - 1);
        let (lo, hi) = stack.pop().unwrap();

        if hi - lo < FALLBACK_QSORT_SMALL_THRESH {
            fallback_simple_sort(fmap, eclass, lo, hi);
            continue;
        }

        /* Random partitioning.  Median of 3 sometimes fails to
           avoid bad cases.  Median of 9 seems to help but
           looks rather expensive.  This too seems to work but
           is cheaper.  Guidance for the magic constants
           7621 and 32768 is taken from Sedgewick's algorithms
           book, chapter 35.
        */
        r = ((r * 7621) + 1) % 32768;
        let r3 = r % 3;
        let med: i32;

        if r3 == 0 {
            med = eclass[fmap[lo] as usize] as i32;
        } else if r3 == 1 {
            med = eclass[fmap[((lo + hi) >> 1)] as usize] as i32;
        } else {
            med = eclass[fmap[hi] as usize] as i32;
        }

        let (mut un_lo, lt_lo) = (lo, lo);
        let (mut un_hi, mut gt_hi) = (hi, hi);

        loop {
            loop {
                if un_lo > un_hi {
                    break;
                }

                let n: i32 = eclass[fmap[un_lo] as usize] as i32 - med;
                if n == 0 {
                    fmap.swap(un_hi, gt_hi);
                    gt_hi -= 1;
                    un_hi -= 1;
                    continue;
                }

                if n < 0 {
                    break;
                }

                un_hi -= 1;
            }

            if un_lo > un_hi {
                break;
            }

            fmap.swap(un_lo, un_hi);
            un_lo += 1;
            un_hi -= 1;
        }

        assert!(un_hi == un_lo - 1, "fallback_qsort_3(2)");

        if gt_hi < lt_lo {
            continue;
        }

        let mut n = min(lt_lo - lo, un_lo - lt_lo);
        swap_regions(fmap, n, lo, un_lo - n);
        let mut m = min(hi - gt_hi, gt_hi - un_hi);
        swap_regions(fmap, m, un_lo, hi - m + 1);

        n = lo + un_lo - lt_lo - 1;
        m = hi - (gt_hi - un_hi) + 1;

        if n - lo > hi - m {
            stack.push((lo, n));
            stack.push((m, hi));
        } else {
            stack.push((m, hi));
            stack.push((lo, n));
        }
    }
}

/// Pre:
///    nblock > 0
///    eclass exists for [0 .. nblock-1]
///    ((UChar*)eclass) [0 .. nblock-1] holds block
///    ptr exists for [0 .. nblock-1]
///
/// Post:
///    ((UChar*)eclass) [0 .. nblock-1] holds block
///    All other areas of eclass destroyed
///    fmap [0 .. nblock-1] holds sorted order
///    bhtab [ 0 .. 2+(nblock/32) ] destroyed
fn fallback_sort(fmap: &mut [u32], eclass: &[u32], bhtab: &[u32], nblock: i32, verb: i32) {
    // let ftab: Vec<i32> = Vec::with_capacity(257);
    // let ftab_copy: Vec<i32> = Vec::with_capacity(256);

    //     UChar* eclass8 = (UChar*)eclass;
    todo!("Somehow figure out a way to interpret &[u32] as &[u8] as well");
}

/// The main, O(N^2 log(N)) sorting algorithm.
/// Faster for "normal" non-repetitive blocks.
fn main_gt_u(
    i1: usize,
    i2: usize,
    block: &[char],
    quadrant: &[u16],
    nblock: usize,
    budget: &mut i32,
) -> bool {
    assert!(i1 != i2, "main_gt_u");
    let (mut i1, mut i2) = (i1, i2);

    for _ in 0..12 {
        let (c1, c2) = (block[i1], block[i2]);

        if c1 != c2 {
            return c1 > c2;
        }

        i1 += 1;
        i2 += 1;
    }

    let mut k = (nblock + 8) as isize;

    while k >= 0 {
        for _ in 0..8 {
            let (c1, c2) = (block[i1], block[i2]);
            if c1 != c2 {
                return c1 > c2;
            }

            let (s1, s2) = (quadrant[i1], quadrant[i2]);
            if s1 != s2 {
                return s1 > s2;
            }

            i1 += 1;
            i2 += 1;
        }

        if i1 >= nblock {
            i1 -= nblock;
        }
        if i2 >= nblock {
            i2 -= nblock;
        }

        k -= 8;
        (*budget) -= 1;
    }

    false
}

fn main_simple_sort(
    ptr: &mut [u32],
    block: &[char],
    quadrant: &[u16],
    nblock: usize,
    lo: usize,
    hi: usize,
    d: usize,
    budget: &mut i32,
) {
    let incs = vec![
        1, 4, 13, 40, 121, 364, 1093, 3280, 9841, 29524, 88573, 265720, 797161, 2391484,
    ];

    let big_n = hi - lo + 1;
    if big_n < 2 {
        return;
    }

    let mut hp: isize = 0;
    while incs[hp as usize] < big_n {
        hp += 1;
    }
    hp -= 1;

    while hp >= 0 {
        let h = incs[hp as usize];
        let mut i = lo + h;

        loop {
            let mut should_break = false;
            for _ in 0..3 {
                if i > hi {
                    should_break = true;
                    break;
                }

                let v = ptr[i];
                let mut j = i;
                while main_gt_u(
                    ptr[j - h] as usize + d,
                    v as usize + d,
                    block,
                    quadrant,
                    nblock,
                    budget,
                ) {
                    ptr[j] = ptr[j - h];
                    j -= h;
                    if j <= (lo + h - 1) {
                        break;
                    }
                }
                ptr[j] = v;
                i += 1;
            }
            if should_break {
                break;
            }

            if *budget < 0 {
                return;
            }
        }

        hp -= 1;
    }
}

/// middle value of (a,b,c)
fn mmed3(a: char, b: char, c: char) -> char {
    max(min(a, b), min(max(a, b), c))
}

/// An implementation of an elegant 3-way quicksort for strings,
/// described in a paper "Fast Algorithms for Sorting and Searching Strings"
/// by Robert Sedgewick and Jon L. Bentley.
fn main_qsort3(
    ptr: &mut [u32],
    block: &[char],
    quadrant: &[u16],
    nblock: usize,
    lo_st: usize,
    hi_st: usize,
    d_st: usize,
    budget: &mut i32,
) {
    const MAIN_QSORT_STACK_SIZE: usize = 100;
    const MAIN_QSORT_SMALL_THRESH: usize = 20;

    // TODO: these should be in the private thing
    //                                   = BZ_N_RADIX + BZ_N_QSORT
    const MAIN_QSORT_DEPTH_THRESH: usize = 2 + 12;

    let mut stack = Vec::with_capacity(MAIN_QSORT_STACK_SIZE);
    stack.push((lo_st, hi_st, d_st));

    while !stack.is_empty() {
        assert!(stack.len() < MAIN_QSORT_STACK_SIZE - 2);
        let (lo, hi, d) = stack.pop().unwrap();

        if hi - lo < MAIN_QSORT_SMALL_THRESH || d > MAIN_QSORT_DEPTH_THRESH {
            main_simple_sort(ptr, block, quadrant, nblock, lo, hi, d, budget);
            if *budget < 0 {
                return;
            }
            continue;
        }

        let med = mmed3(
            block[ptr[lo] as usize + d],
            block[ptr[hi] as usize + d],
            block[ptr[(lo + hi) >> 1] as usize + d],
        ) as i32;

        let (mut un_lo, mut lt_lo) = (lo, lo);
        let (mut un_hi, mut gt_hi) = (hi, hi);

        loop {
            loop {
                if un_lo > un_hi {
                    break;
                }
                let n = block[ptr[un_lo] as usize + d] as i32 - med;

                if n == 0 {
                    ptr.swap(un_lo, lt_lo);
                    lt_lo += 1;
                    un_lo += 1;
                    continue;
                }

                if n > 0 {
                    break;
                }

                un_lo += 1;
            }

            loop {
                if un_lo > un_hi {
                    break;
                }

                let n = block[ptr[un_hi] as usize + d] as i32 - med;
                if n == 0 {
                    ptr.swap(un_hi, gt_hi);
                    gt_hi -= 1;
                    un_hi -= 1;
                    continue;
                }

                if n < 0 {
                    break;
                }

                un_hi -= 1;
            }

            if un_lo > un_hi {
                break;
            }

            ptr.swap(un_lo, un_hi);
            un_lo += 1;
            un_hi -= 1;
        }

        assert!(un_hi == un_lo - 1, "main_qsort3(2)");

        if gt_hi < lt_lo {
            stack.push((lo, hi, d + 1));
            continue;
        }

        let n = min(lt_lo - lo, un_lo - lt_lo);
        swap_regions(ptr, lo, un_lo - n, n);

        let m = min(hi - gt_hi, gt_hi - un_hi);
        swap_regions(ptr, m, un_lo, hi - m + 1);

        #[inline]
        fn next_size(next: &[(usize, usize, usize)], i: usize) -> usize {
            next[i].1 - next[i].0
        }

        let mut next = vec![(lo, n, d), (m, hi, d), (n + 1, m - 1, d + 1)];
        for (x, y) in vec![(0, 1), (1, 2), (0, 1)] {
            if next_size(&next, x) < next_size(&next, y) {
                next.swap(x, y);
            }
        }

        assert!(next_size(&next, 0) >= next_size(&next, 1), "main_qsort3(8)");
        assert!(next_size(&next, 1) >= next_size(&next, 2), "main_qsort3(9)");

        for i in 0..3 {
            stack.push(next[i]);
        }
    }
}

/// Pre:
///    nblock > N_OVERSHOOT
///    block32 exists for [0 .. nblock-1 +N_OVERSHOOT]
///    ((UChar*)block32) [0 .. nblock-1] holds block
///    ptr exists for [0 .. nblock-1]
///
/// Post:
///    ((UChar*)block32) [0 .. nblock-1] holds block
///    All other areas of block32 destroyed
///    ftab [0 .. 65536 ] destroyed
///    ptr [0 .. nblock-1] holds sorted order
///    if (*budget < 0), sorting was abandoned
fn main_sort(
    ptr: &mut [u32],
    block: &mut [char],
    quadrant: &mut [u16],
    ftab: &mut [usize; TWO_BYTE_RANGE],
    nblock: usize,
    budget: &mut i32,
) -> bool {
    const SETMASK: u32 = 1 << 21;
    const CLEARMASK: u32 = !SETMASK;
    const BZ_N_RADIX: usize = 2;

    // set up the 2-byte frequency table
    for elm in ftab.iter_mut() {
        *elm = 0;
    }

    let mut j = ((block[0] as u16) << 8) as u32;
    let mut i = nblock - 1;

    while i >= 3 {
        quadrant[i - 0] = 0;
        j = (j >> 8) | ((block[i - 0] as u16) << 8) as u32;
        ftab[j as usize] += 1;

        quadrant[i - 1] = 0;
        j = (j >> 8) | ((block[i - 1] as u16) << 8) as u32;
        ftab[j as usize] += 1;

        quadrant[i - 2] = 0;
        j = (j >> 8) | ((block[i - 2] as u16) << 8) as u32;
        ftab[j as usize] += 1;

        quadrant[i - 3] = 0;
        j = (j >> 8) | ((block[i - 3] as u16) << 8) as u32;
        ftab[j as usize] += 1;

        i -= 4;
    }

    while i >= 0 {
        quadrant[i] = 0;
        j = (j >> 8) | ((block[i] as u16) << 8) as u32;
        ftab[j as usize] += 1;
        i -= 1;
    }

    for i in 0..N_OVERSHOOT {
        block[nblock + i] = block[i];
        quadrant[nblock + i] = 0;
    }

    for i in 1..=TWO_BYTE_RANGE {
        ftab[i] += ftab[i - 1];
    }

    let mut s = (block[0] as u16) << 8;
    let mut i = nblock - 1;

    while i >= 3 {
        s = (s >> 8) | ((block[i - 0] as u16) << 8);
        let j = ftab[s as usize] - 1;
        ftab[s as usize] = j;
        ptr[j as usize] = i as u32 - 0;

        s = (s >> 8) | ((block[i - 1] as u16) << 8);
        let j = ftab[s as usize] - 1;
        ftab[s as usize] = j;
        ptr[j as usize] = i as u32 - 1;

        s = (s >> 8) | ((block[i - 2] as u16) << 8);
        let j = ftab[s as usize] - 1;
        ftab[s as usize] = j;
        ptr[j as usize] = i as u32 - 2;

        s = (s >> 8) | ((block[i - 3] as u16) << 8);
        let j = ftab[s as usize] - 1;
        ftab[s as usize] = j;
        ptr[j as usize] = i as u32 - 3;

        i -= 4;
    }

    while i >= 0 {
        s = (s >> 8) | ((block[i] as u16) << 8);
        let j = ftab[s as usize] - 1;
        ftab[s as usize] = j;
        ptr[j as usize] = i as u32;
        i -= 1;
    }

    /*--
       Now ftab contains the first loc of every small bucket.
       Calculate the running order, from smallest to largest
       big bucket.
    --*/
    let mut big_done = vec![false; BYTE_RANGE];
    let mut running_order: Vec<usize> = (0..BYTE_RANGE).collect();

    {
        // TODO: what if the subtraction is negative?
        fn big_freq(ftab: &mut [usize], b: usize) -> usize {
            ftab[(b + 1) << 8] - ftab[b << 8]
        }

        // h = 1; do { h = 3*h + 1 } while (h <= 256)
        let mut h = 364;
        let mut vv = 0;

        while h != 1 {
            h /= 3;
            for i in h..BYTE_RANGE {
                vv = running_order[i];
                let mut j = i;

                let mut leave_loop = false;

                while big_freq(ftab, j - h) > big_freq(ftab, vv) {
                    running_order[j] = running_order[j - h];
                    j -= h;

                    if j <= (h - 1) {
                        leave_loop = true;
                        break;
                    }
                }

                if leave_loop {
                    break;
                }
            }

            // TODO: check if this `j` is the intended indexer
            running_order[j] = vv;
        }
    }

    /*--
       The main sorting loop.
    --*/

    let mut num_q_sorted = 0;

    for i in 0..BYTE_RANGE {
        /*--
           Process big buckets, starting with the least full.
           Basically this is a 3-step process in which we call
           mainQSort3 to sort the small buckets [ss, j], but
           also make a big effort to avoid the calls if we can.
        --*/
        let ss = running_order[i];

        /*--
           Step 1:
           Complete the big bucket [ss] by quicksorting
           any unsorted small buckets [ss, j], for j != ss.
           Hopefully previous pointer-scanning phases have already
           completed many of the small buckets [ss, j], so
           we don't have to sort them at all.
        --*/
        for j in 0..256 {
            if j != ss {
                let sb = (ss << 8) + j;
                if ftab[sb] & SETMASK == 0 {
                    let lo = (ftab[sb] & CLEARMASK);
                    // TODO: check never negative
                    let hi = (ftab[sb + 1] & CLEARMASK) - 1;

                    if hi > lo {
                        main_qsort3(ptr, block, quadrant, nblock, lo, hi, BZ_N_RADIX, budget);
                        num_q_sorted += hi + 1 - lo;
                        if *budget < 0 {
                            return false;
                        }
                    }
                }
                ftab[sb] |= SETMASK;
            }
        }

        assert!(!big_done[ss]);

        /*--
           Step 2:
           Now scan this big bucket [ss] so as to synthesise the
           sorted order for small buckets [t, ss] for all t,
           including, magically, the bucket [ss,ss] too.
           This will avoid doing Real Work in subsequent Step 1's.
        --*/
        {
            let mut copy_start = [0; BYTE_RANGE];
            let mut copy_end = [0; BYTE_RANGE];

            for (j, (start, end)) in copy_start.iter_mut().zip(copy_end.iter_mut()).enumerate() {
                *start = ftab[(j << 8) + ss] & CLEARMASK;
                *end = (ftab[(j << 8) + ss + 1] & CLEARMASK) - 1;
            }

            for j in (ftab[ss << 8] & CLEARMASK)..(copy_start[ss]) {
                let mut k = ptr[j] as isize - 1;
                if k < 0 {
                    k += nblock as isize;
                }

                let c1 = block[k as usize] as usize;
                if !big_done[c1] {
                    ptr[copy_start[c1] as usize] = k as u32;
                    copy_start[c1] += 1;
                }
            }

            for j in (ftab[(ss + 1) << 8] & CLEARMASK - 1)..(copy_end[ss]) {
                let mut k = ptr[j] as isize - 1;
                if k < 0 {
                    k += nblock as isize;
                }

                let c1 = block[k as usize] as usize;

                if !big_done[c1] {
                    ptr[copy_end[c1] as usize] = k as u32;
                    copy_end[c1] -= 1;
                }
            }

            assert!(
                copy_start[ss] - 1 == copy_end[ss]
                    /* Extremely rare case missing in bzip2-1.0.0 and 1.0.1.
                       Necessity for this case is demonstrated by compressing
                       a sequence of approximately 48.5 million of character
                       251; 1.0.0/1.0.1 will then die here.
                     */
                    || (copy_start[ss] == 0
                    && copy_end[ss] == (nblock as u32 - 1))
            );
        }

        for j in 0..BYTE_RANGE {
            ftab[(j << 8) + ss] |= SETMASK;
        }

        /*--
           Step 3:
           The [ss] big bucket is now done.  Record this fact,
           and update the quadrant descriptors.  Remember to
           update quadrants in the overshoot area too, if
           necessary.  The "if (i < 255)" test merely skips
           this updating for the last bucket processed, since
           updating for the last bucket is pointless.

           The quadrant array provides a way to incrementally
           cache sort orderings, as they appear, so as to
           make subsequent comparisons in fullGtU() complete
           faster.  For repetitive blocks this makes a big
           difference (but not big enough to be able to avoid
           the fallback sorting mechanism, exponential radix sort).

           The precise meaning is: at all times:

              for 0 <= i < nblock and 0 <= j <= nblock

              if block[i] != block[j],

                 then the relative values of quadrant[i] and
                      quadrant[j] are meaningless.

                 else {
                    if quadrant[i] < quadrant[j]
                       then the string starting at i lexicographically
                       precedes the string starting at j

                    else if quadrant[i] > quadrant[j]
                       then the string starting at j lexicographically
                       precedes the string starting at i

                    else
                       the relative ordering of the strings starting
                       at i and j has not yet been determined.
                 }
        --*/

        big_done[ss] = true;

        if i < 255 {
            // TODO: this bitwise & with these two variables repeats a lot
            // might be able to optimize this code duplication
            let bb_start = (ftab[ss << 8] & CLEARMASK);
            let bb_size = (ftab[(ss + 1) << 8] & CLEARMASK) - bb_start;

            let mut shifts = 0;

            while (bb_size >> shifts) > 65534 {
                shifts += 1;
            }

            for j in (bb_size - 1)..=0 {
                let a2update = ptr[bb_start + j] as usize;
                let q_val = (j >> shifts) as u16;
                quadrant[a2update] = q_val;

                if a2update < N_OVERSHOOT {
                    quadrant[a2update + nblock] = q_val;
                }
            }

            assert!(((bb_size - 1) >> shifts) <= 65535);
        }
    }

    true
}

/* Pre:
      nblock > 0
      arr2 exists for [0 .. nblock-1 +N_OVERSHOOT]
      ((UChar*)arr2)  [0 .. nblock-1] holds block
      arr1 exists for [0 .. nblock-1]

   Post:
      ((UChar*)arr2) [0 .. nblock-1] holds block
      All other areas of block destroyed
      ftab [ 0 .. 65536 ] destroyed
      arr1 [0 .. nblock-1] holds sorted order
*/
pub fn block_sort(buf: &[u8], ftab: &mut [u32; TWO_BYTE_RANGE], work_factor: u8) {
    const LOWER_LIMIT: usize = 100000;
    let mut use_fallback = true;

    if buf.len() >= LOWER_LIMIT {
        let mut i = buf.len() + N_OVERSHOOT;
        if (i & 1) == 1 {
            i += 1;
        }

        let quadrant = &buf[i];

        let budget_init = buf.len() * ((work_factor - 1) / 3) as usize;
        let budget = budget_init;

        let passed = main_sort(buf, quadrant, ftab, buf.len(), budget);

        if passed {
            use_fallback = false;
        }
    }

    if use_fallback {
        fallback_sort();
    }
}
