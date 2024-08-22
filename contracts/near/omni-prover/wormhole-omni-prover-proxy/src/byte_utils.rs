pub trait ByteUtils {
    fn get_u8(&self, index: usize) -> u8;
    fn get_u16(&self, index: usize) -> u16;
    fn get_u32(&self, index: usize) -> u32;
    fn get_u64(&self, index: usize) -> u64;

    fn get_u128_be(&self, index: usize) -> u128;
    /// High 128 then low 128
    fn get_u256(&self, index: usize) -> (u128, u128);
    //    fn get_address(&self, index: usize) -> CanonicalAddr;
    fn get_bytes32(&self, index: usize) -> &[u8];
    fn get_bytes(&self, index: usize, bytes: usize) -> &[u8];
    fn get_const_bytes<const N: usize>(&self, index: usize) -> [u8; N];
}

impl ByteUtils for &[u8] {
    fn get_u8(&self, index: usize) -> u8 {
        self[index]
    }
    fn get_u16(&self, index: usize) -> u16 {
        let mut bytes: [u8; 16 / 8] = [0; 16 / 8];
        bytes.copy_from_slice(&self[index..index + 2]);
        u16::from_be_bytes(bytes)
    }
    fn get_u32(&self, index: usize) -> u32 {
        let mut bytes: [u8; 32 / 8] = [0; 32 / 8];
        bytes.copy_from_slice(&self[index..index + 4]);
        u32::from_be_bytes(bytes)
    }
    fn get_u64(&self, index: usize) -> u64 {
        let mut bytes: [u8; 64 / 8] = [0; 64 / 8];
        bytes.copy_from_slice(&self[index..index + 8]);
        u64::from_be_bytes(bytes)
    }
    fn get_u128_be(&self, index: usize) -> u128 {
        let mut bytes: [u8; 128 / 8] = [0; 128 / 8];
        bytes.copy_from_slice(&self[index..index + 128 / 8]);
        u128::from_be_bytes(bytes)
    }
    fn get_u256(&self, index: usize) -> (u128, u128) {
        (self.get_u128_be(index), self.get_u128_be(index + 128 / 8))
    }
    //    fn get_address(&self, index: usize) -> CanonicalAddr {
    // 32 bytes are reserved for addresses, but only the last 20 bytes are taken by the actual address
    //        CanonicalAddr::from(&self[index + 32 - 20..index + 32])
    //    }
    fn get_bytes32(&self, index: usize) -> &[u8] {
        &self[index..index + 32]
    }

    fn get_bytes(&self, index: usize, bytes: usize) -> &[u8] {
        &self[index..index + bytes]
    }

    fn get_const_bytes<const N: usize>(&self, index: usize) -> [u8; N] {
        let mut bytes: [u8; N] = [0; N];
        bytes.copy_from_slice(&self[index..index + N]);
        bytes
    }
}
