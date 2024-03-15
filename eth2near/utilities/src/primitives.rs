#[derive(Debug, Clone)]
pub struct FixedBytes<const N: usize>(pub [u8; N]);
#[derive(Debug, Clone)]
pub struct Bytes(pub Vec<u8>);
#[derive(Debug, Clone)]
pub struct Byte(pub u8);

pub type U256 = FixedBytes<32>;
pub type Address = FixedBytes<20>;
pub type Bloom = FixedBytes<256>;
