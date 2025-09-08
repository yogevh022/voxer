#[allow(dead_code)]
mod fragmentation;
mod net;
mod traits;
mod types;

pub use net::UdpChannel;
pub use types::{MessageBytes, NetworkMessageTag, ReceivedMessage, SerializedMessage};
pub use traits::{NetworkSerializable, NetworkDeserializable, NetworkMessageConfig};

#[derive(Debug)]
pub enum NetworkingError {
    SocketError,
    InvalidChecksum,
}
