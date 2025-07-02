use std::error::Error;

#[derive(Debug, Default)]
pub struct GetStatusRequestV0 {}

impl GetStatusRequestV0 {
    pub fn new() -> Self {
        GetStatusRequestV0 {}
    }
}

// Трейт для сериализации
pub trait BinaryMarshaler {
    fn marshal_binary(&self) -> Result<Vec<u8>, Box<dyn Error>>;
}

impl BinaryMarshaler for GetStatusRequestV0 {
    fn marshal_binary(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(vec![0])
    }
}