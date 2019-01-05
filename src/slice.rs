pub fn pack_in_u64(offset: usize, length: usize) -> u64 {
    (length as u64) | ((offset as u64) << 32)
}

pub fn unpack_from_u64(input: u64) -> (usize, usize) {
    (((input & 0xFFFF_FFFF_0000_0000) >> 32) as usize, (input & 0x0000_0000_FFFF_FFFF) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn slice_encoding_roundtrip(i in any::<u32>(), l in any::<u32>()) {
            prop_assert_eq!((i as usize, l as usize), unpack_from_u64(pack_in_u64(i as usize, l as usize)))
        }
    }
}
