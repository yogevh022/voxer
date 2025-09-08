use bytemuck::Pod;
use crate::fragmentation::fragment_bytes;
use crate::types::{NetworkMessageTag, SerializedMessage};

pub trait NetworkMessageConfig {
    fn tag(&self) -> NetworkMessageTag;
    fn fragment_count(&self) -> usize;
    fn tag_size(&self) -> usize {
        size_of::<NetworkMessageTag>()
    }
}

pub trait NetworkSerializable: NetworkMessageConfig {
    fn serialize(&self) -> SerializedMessage;
}

impl<T> NetworkSerializable for T
where
    T: Pod + NetworkMessageConfig,
{
    fn serialize(&self) -> SerializedMessage {
        let bytes = bytemuck::bytes_of(self);
        match self.fragment_count() {
            1 => {
                let mut data = Vec::with_capacity(size_of::<Self>() + self.tag_size());
                data.extend_from_slice(bytes);
                data.extend(self.tag().to_be_bytes());
                SerializedMessage::Single(data)
            }
            _ => {
                let fragmented_bytes = fragment_bytes(bytes, self.fragment_count(), self.tag());
                SerializedMessage::Fragmented(fragmented_bytes)
            }
        }
    }
}

pub trait NetworkDeserializable: NetworkMessageConfig {
    fn deserialize<B: AsRef<[u8]>>(data: B) -> Self;
}

impl<T> NetworkDeserializable for T
where
    T: Sized + Pod + NetworkMessageConfig,
{
    fn deserialize<B: AsRef<[u8]>>(data: B) -> Self {
        *bytemuck::try_from_bytes(data.as_ref()).unwrap()
    }
}
