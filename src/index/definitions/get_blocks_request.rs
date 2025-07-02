use std::error::Error;
use eosio_shipper_gf::shipper_types::BlockPosition;
use crate::index::definitions::get_status_request::BinaryMarshaler;

#[derive(Debug)]
pub struct GetBlocksRequestV0 {
    pub start_block_num: u32,
    pub end_block_num: u32,
    pub max_messages_in_flight: u32,
    pub have_positions: Vec<BlockPosition>,
    pub irreversible_only: bool,
    pub fetch_block: bool,
    pub fetch_traces: bool,
    pub fetch_deltas: bool,
}

impl BinaryMarshaler for GetBlocksRequestV0 {
    fn marshal_binary(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buf = Vec::new();
        buf.push(1); // Тег для GetBlocksRequestV0

        buf.extend(&self.start_block_num.to_le_bytes());
        buf.extend(&self.end_block_num.to_le_bytes());
        buf.extend(&self.max_messages_in_flight.to_le_bytes());

        // Кодирование have_positions
        encode_varuint32(self.have_positions.len() as u32, &mut buf);
        for pos in &self.have_positions {
            buf.extend(&pos.block_num.to_le_bytes());
            buf.extend(pos.block_id.as_bytes());
        }

        buf.push(self.irreversible_only as u8);
        buf.push(self.fetch_block as u8);
        buf.push(self.fetch_traces as u8);
        buf.push(self.fetch_deltas as u8);

        Ok(buf)
    }
}

fn encode_varuint32(mut v: u32, buf: &mut Vec<u8>) {
    loop {
        let mut byte = (v & 0x7F) as u8;
        v >>= 7;
        if v != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if v == 0 {
            break;
        }
    }
}