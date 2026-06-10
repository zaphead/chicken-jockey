use crate::messages::{ClientPacket, ServerPacket};
use crate::PROTOCOL_VERSION;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecError {
    Empty,
    VersionMismatch,
    Decode,
}

pub fn encode_server_packet(packet: &ServerPacket) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
    bytes.extend(bincode::serialize(packet).expect("serialize server packet"));
    bytes
}

pub fn encode_client_packet(packet: &ClientPacket) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&PROTOCOL_VERSION.to_le_bytes());
    bytes.extend(bincode::serialize(packet).expect("serialize client packet"));
    bytes
}

pub fn decode_server_packet(bytes: &[u8]) -> Result<ServerPacket, CodecError> {
    decode_packet(bytes)
}

pub fn decode_client_packet(bytes: &[u8]) -> Result<ClientPacket, CodecError> {
    decode_packet(bytes)
}

pub fn client_packet_uses_datagram(packet: &ClientPacket) -> bool {
    matches!(packet, ClientPacket::Input(_))
}

pub fn server_packet_uses_datagram(packet: &ServerPacket) -> bool {
    matches!(packet, ServerPacket::EntitySnapshots(_))
}

fn decode_packet<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, CodecError> {
    if bytes.len() < 4 {
        return Err(CodecError::Empty);
    }
    let version = u32::from_le_bytes(bytes[0..4].try_into().expect("version bytes"));
    if version != PROTOCOL_VERSION {
        return Err(CodecError::VersionMismatch);
    }
    bincode::deserialize(&bytes[4..]).map_err(|_| CodecError::Decode)
}
