#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode))]
pub struct Created {}
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "musli", derive(musli::Encode, musli::Decode))]
pub struct Ok {}

pub trait DataResponseStatus {
    fn status() -> u16;
}

impl DataResponseStatus for Created {
    fn status() -> u16 {
        201
    }
}

impl DataResponseStatus for Ok {
    fn status() -> u16 {
        200
    }
}
