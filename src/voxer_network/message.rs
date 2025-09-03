use std::net::SocketAddr;
use std::sync::atomic::AtomicU32;

pub type MessageTagType = u16;
const FRAGMENTED_TAG: MessageTagType = MessageTagType::MAX;
static FRAGMENT_ID_COUNTER: AtomicU32 = AtomicU32::new(0);
fn next_fragment_id() -> u32 {
    FRAGMENT_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[repr(C)]
#[derive(Debug)]
pub struct NetworkMessageFragment {
    pub id: u32,
    pub index: u8,
    pub total: u8,
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
    const FRAGMENT_SIZE: usize = size_of::<Self>() / Self::FRAGMENT_COUNT;

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
            _ => NetworkSendMessage::Fragmented(fragment_bytes(
                Self::TAG,
                Self::FRAGMENT_COUNT,
                Self::FRAGMENT_SIZE,
                bytes,
            )),
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
                let fragment_id = u32::from_be_bytes(
                    self.data[tag_size..(tag_size + size_of::<u32>())]
                        .try_into()
                        .unwrap(),
                );
                let fragment_index = self.data[tag_size + size_of::<u32>()];
                let fragment_total = self.data[tag_size + size_of::<u32>() + size_of::<u8>()];
                NetworkReceiveMessage::Fragment(NetworkMessageFragment {
                    id: fragment_id,
                    index: fragment_index,
                    total: fragment_total,
                    data: self.data
                        [(tag_size + size_of::<NetworkMessageFragment>() - size_of::<Vec<u8>>())..]
                        .to_vec(),
                })
            }
            _ => NetworkReceiveMessage::Single(self.data.to_vec()),
        }
    }
}

fn fragment_bytes(
    inner_tag: MessageTagType,
    fragment_count: usize,
    fragment_size: usize,
    whole_bytes: &[u8],
) -> Vec<Vec<u8>> {
    debug_assert_eq!(
        (whole_bytes.len() + size_of::<MessageTagType>()) % fragment_count,
        0
    );
    let mut fragments = Vec::with_capacity(fragment_count);
    for i in 0..fragment_count {
        let mut data = Vec::with_capacity(size_of::<MessageTagType>() + fragment_size);
        data.extend(FRAGMENTED_TAG.to_be_bytes());
        if i == 0 {
            data.extend(inner_tag.to_be_bytes());
        }
        data.extend_from_slice(next_fragment_id().to_be_bytes().as_slice());
        data.push(i as u8);
        data.push(fragment_count as u8);
        data.extend_from_slice(&whole_bytes[i * fragment_size..(i + 1) * fragment_size]);
        fragments.push(data);
    }

    fragments
}
