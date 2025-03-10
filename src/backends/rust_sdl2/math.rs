use std::num::NonZeroU16;

pub fn capped_next_power_of_two(n: NonZeroU16) -> NonZeroU16 {
    let leading = n.leading_zeros();
    let trailing = n.trailing_zeros();
    // 0000 0000 1000 0000
    if leading + trailing >= 15 {
        // power of two detected. return as is.
        // safety - will return 0 only if 0 is the input
        return n;
    }

    // safety - clearly every branch gives non zero
    unsafe {
        NonZeroU16::new_unchecked(match leading {
            // 0000 0000 0000 0000
            // 16 => <handled above>
            // 0000 0000 0000 0001
            // 15 => <handled above>
            // 0000 0000 0000 0011
            14 => 4,
            // 0000 0000 0000 0111
            13 => 8,
            // 0000 0000 0000 1111
            12 => 16,
            // 0000 0000 0001 1111
            11 => 32,
            10 => 64,
            9 => 128,
            8 => 256,
            7 => 512,
            6 => 1024,
            5 => 2048,
            // even for large numbers, don't go too high with the point size.
            // this size of font would be ridiculous
            _ => 4096,
        })
    }
}
