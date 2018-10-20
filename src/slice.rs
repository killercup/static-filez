pub fn pack_in_u64(offset: usize, len: usize) -> u64 {
    let mut res = 0_u64;
    res |= offset << 32;
    res |= length;
    res
}

pub fn unpack_from_u64(input: u64) -> (usize, usize) {
    (
        (input & 0xFFFF_FFFF_0000_0000 >> 32) as usize,
        (input & 0x0000_0000_FFFF_FFFF) as usize,
    )
}
