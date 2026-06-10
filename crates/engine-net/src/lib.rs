//! Network protocol definitions and QUIC transport.

mod cert;
mod codec;
mod messages;
mod transport;

pub use codec::{
    decode_client_packet, decode_server_packet, encode_client_packet, encode_server_packet,
    CodecError,
};
pub use messages::{
    BlockDelta, ClientPacket, EntitySnapshot, PlayerInput, ServerPacket, DEFAULT_PORT,
};
pub use transport::{NetClient, NetServer};

pub const PROTOCOL_VERSION: u32 = 1;
