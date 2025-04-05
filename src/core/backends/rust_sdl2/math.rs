use std::num::NonZeroU16;

pub fn capped_next_power_of_two(n: NonZeroU16) -> NonZeroU16 {
    let leading = n.leading_zeros();
    let trailing = n.trailing_zeros();
    // 0000 0000 1000 0000
    if leading + trailing >= 15 {
        // power of two detected. return as is.
        return n;
    }

    // safety - each branch literally states nonzero
    unsafe {
        NonZeroU16::new_unchecked(match leading {
            // 0000 0000 0000 0000
            // 16 => <never since nonzero>
            // 0000 0000 0000 0001
            // 15 => <handled above since 15 leading zeros in u16 type is 1, which is a power of 2>
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(
            capped_next_power_of_two(1.try_into().unwrap()),
            1.try_into().unwrap()
        );
        assert_eq!(
            capped_next_power_of_two(2.try_into().unwrap()),
            2.try_into().unwrap()
        );
        assert_eq!(
            capped_next_power_of_two(3.try_into().unwrap()),
            4.try_into().unwrap()
        );
        assert_eq!(
            capped_next_power_of_two(4.try_into().unwrap()),
            4.try_into().unwrap()
        );
        assert_eq!(
            capped_next_power_of_two(5.try_into().unwrap()),
            8.try_into().unwrap()
        );
        assert_eq!(
            capped_next_power_of_two(6.try_into().unwrap()),
            8.try_into().unwrap()
        );
        assert_eq!(
            capped_next_power_of_two(7.try_into().unwrap()),
            8.try_into().unwrap()
        );
        assert_eq!(
            capped_next_power_of_two(8.try_into().unwrap()),
            8.try_into().unwrap()
        );
        assert_eq!(
            capped_next_power_of_two(9.try_into().unwrap()),
            16.try_into().unwrap()
        );
        assert_eq!(
            capped_next_power_of_two(10.try_into().unwrap()),
            16.try_into().unwrap()
        );
        assert_eq!(
            capped_next_power_of_two(12.try_into().unwrap()),
            16.try_into().unwrap()
        );
        assert_eq!(
            capped_next_power_of_two(0xFFFF.try_into().unwrap()),
            4096.try_into().unwrap()
        );
    }
}
