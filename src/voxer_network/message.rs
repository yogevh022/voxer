use std::net::SocketAddr;
use std::sync::atomic::AtomicU32;

pub type MessageTagType = u8;
const FRAGMENTED_TAG: MessageTagType = MessageTagType::MAX;
static FRAGMENT_ID_COUNTER: AtomicU32 = AtomicU32::new(0);
fn next_fragment_id() -> u32 {
    FRAGMENT_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct NetworkMessageFragmentHeader {
    pub tag: MessageTagType,
    pub index: u8,
    pub total: u8,
    pub id: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct NetworkMessageFragment {
    pub header: NetworkMessageFragmentHeader,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct NetworkMessage {
    pub src: SocketAddr,
    pub data: Vec<u8>,
}

impl NetworkMessage {
    pub fn new(src: SocketAddr, data: Vec<u8>) -> Self {
        Self { src, data }
    }
}

pub struct NetworkRawMessage<'a> {
    pub data: &'a [u8],
}

impl<'a> NetworkRawMessage<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

pub enum NetworkSendMessage {
    Single(Vec<u8>),
    Fragmented(Vec<Vec<u8>>),
}

pub enum NetworkReceiveMessage {
    Single(Vec<u8>),
    Fragment(NetworkMessageFragment),
}

/// The default impl requires `repr(C)` and `Pod`
pub trait NetworkSerializable: Sized {
    const TAG: MessageTagType;
    const FRAGMENT_COUNT: usize = 1;
    const TAG_SIZE: usize = size_of_val(&Self::TAG);

    fn serialize(&self) -> NetworkSendMessage {
        let bytes = unsafe {
            std::slice::from_raw_parts((self as *const Self) as *const u8, size_of::<Self>())
        };
        match Self::FRAGMENT_COUNT {
            0 => {
                panic!("FRAGMENT_COUNT must be greater than 0");
            }
            1 => {
                let mut data = Vec::with_capacity(Self::TAG_SIZE + size_of::<Self>());
                data.extend(Self::TAG.to_be_bytes());
                data.extend_from_slice(bytes);
                NetworkSendMessage::Single(data)
            }
            _ => {
                let fragment_id = next_fragment_id();
                NetworkSendMessage::Fragmented(fragment_bytes(
                    Self::TAG,
                    fragment_id,
                    Self::FRAGMENT_COUNT,
                    bytes,
                ))
            }
        }
    }
}

pub trait NetworkDeserializable: Sized {
    fn deserialize(self) -> NetworkReceiveMessage;
}

impl<'a> NetworkDeserializable for NetworkRawMessage<'a> {
    fn deserialize(self) -> NetworkReceiveMessage {
        let tag_size = size_of::<MessageTagType>();
        let tag = MessageTagType::from_be_bytes(self.data[0..tag_size].try_into().unwrap());
        match tag {
            FRAGMENTED_TAG => {
                let fragment_index = self.data[tag_size];
                let fragment_total = self.data[tag_size + 1];
                let fragment_id = u32::from_be_bytes(
                    self.data[(tag_size + 1 + 1) // extra 1 to align u32 to 4 bytes
                        ..(tag_size + 1 + 1 + size_of::<u32>())]
                        .try_into()
                        .unwrap(),
                );
                let header = NetworkMessageFragmentHeader {
                    tag,
                    id: fragment_id,
                    index: fragment_index,
                    total: fragment_total,
                };
                NetworkReceiveMessage::Fragment(NetworkMessageFragment {
                    header,
                    data: self.data[(tag_size + size_of::<NetworkMessageFragmentHeader>())..]
                        .to_vec(),
                })
            }
            _ => NetworkReceiveMessage::Single(self.data.to_vec()),
        }
    }
}

fn fragment_bytes(
    inner_tag: MessageTagType,
    fragment_id: u32,
    fragment_count: usize,
    whole_bytes: &[u8],
) -> Vec<Vec<u8>> {
    // todo clean this entire function
    let whole_size = whole_bytes.len() + size_of::<MessageTagType>();
    let fragment_data_size = whole_size / fragment_count;
    let fragment_remainder_size = whole_size % fragment_count;
    let fragment_full_size = fragment_data_size + size_of::<NetworkMessageFragmentHeader>();
    let fragment_remainder_offset = fragment_remainder_size - size_of::<MessageTagType>();

    let mut fragments = Vec::with_capacity(1 + fragment_count);
    let fragment_id_slice = &fragment_id.to_be_bytes();

    // insert remainder as index 0, so we have extra space for the inner tag (even if remainder is 0)
    let mut data =
        Vec::with_capacity(fragment_remainder_size + size_of::<NetworkMessageFragmentHeader>());
    data.extend(FRAGMENTED_TAG.to_be_bytes());
    data.push(0u8);
    data.push(1 + fragment_count as u8);
    data.push(0u8); // padding
    data.extend_from_slice(fragment_id_slice);
    data.extend(inner_tag.to_be_bytes());
    data.extend_from_slice(&whole_bytes[0..fragment_remainder_offset]);
    fragments.push(data);

    for i in 0..fragment_count {
        let mut data = Vec::with_capacity(fragment_full_size);
        data.extend(FRAGMENTED_TAG.to_be_bytes());
        data.push(1 + i as u8);
        data.push(1 + fragment_count as u8);
        data.push(0u8); // padding
        data.extend_from_slice(fragment_id_slice);
        data.extend_from_slice(
            &whole_bytes[(i * fragment_data_size) + fragment_remainder_offset
                ..((i + 1) * fragment_data_size) + fragment_remainder_offset],
        );
        fragments.push(data);
    }

    fragments
}
