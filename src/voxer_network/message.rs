use std::net::SocketAddr;
use std::sync::atomic::AtomicU32;

pub type MessageTagType = u8;
const FRAGMENTED_TAG: MessageTagType = MessageTagType::MAX;
const FRAGMENTED_TAG_BYTES: [u8; size_of::<MessageTagType>()] = FRAGMENTED_TAG.to_be_bytes();
static FRAGMENT_ID_COUNTER: AtomicU32 = AtomicU32::new(0);
fn next_fragment_id() -> u32 {
    FRAGMENT_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct NetworkMessageFragmentTail {
    pub id: u32,
    pub index: u8,
    pub total: u8,
    _pad: u8, // make sure tag is the last byte
    pub tag: MessageTagType,
}

impl NetworkMessageFragmentTail {
    pub fn new(id: u32, index: u8, total: u8, tag: MessageTagType) -> Self {
        Self {
            id,
            index,
            total,
            _pad: 0,
            tag,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct NetworkMessageFragment {
    pub data: Vec<u8>,
    pub tail: NetworkMessageFragmentTail,
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
    const _TAG_SIZE: usize = size_of_val(&Self::TAG);

    fn serialize(&self) -> NetworkSendMessage {
        let bytes = unsafe {
            std::slice::from_raw_parts((self as *const Self) as *const u8, size_of::<Self>())
        };
        match Self::FRAGMENT_COUNT {
            0 => {
                panic!("FRAGMENT_COUNT must be greater than 0");
            }
            1 => {
                let mut data = Vec::with_capacity(size_of::<Self>() + Self::_TAG_SIZE);
                data.extend_from_slice(bytes);
                data.extend(Self::TAG.to_be_bytes());
                NetworkSendMessage::Single(data)
            }
            _ => {
                let fragment_id = next_fragment_id();
                NetworkSendMessage::Fragmented(fragment_bytes(
                    fragment_id,
                    bytes,
                    Self::FRAGMENT_COUNT,
                    Self::TAG,
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
        let tag_end = self.data.len() - size_of::<MessageTagType>();
        let tag =
            MessageTagType::from_be_bytes(self.data[tag_end..self.data.len()].try_into().unwrap());
        match tag {
            FRAGMENTED_TAG => {
                let fragment_total = self.data[tag_end - 2];
                let fragment_index = self.data[tag_end - 3];
                let fragment_id =
                    u32::from_be_bytes(self.data[(tag_end - 7)..(tag_end - 3)].try_into().unwrap());
                let header = NetworkMessageFragmentTail::new(
                    fragment_id,
                    fragment_index,
                    fragment_total,
                    tag,
                );
                NetworkReceiveMessage::Fragment(NetworkMessageFragment {
                    tail: header,
                    data: self.data[..self.data.len() - size_of::<NetworkMessageFragmentTail>()]
                        .to_vec(),
                })
            }
            _ => NetworkReceiveMessage::Single(self.data.to_vec()),
        }
    }
}

fn fragment_bytes(
    fragment_id: u32,
    whole_bytes: &[u8],
    frag_count: usize,
    inner_tag: MessageTagType,
) -> Vec<Vec<u8>> {
    let whole_size = whole_bytes.len() + size_of::<MessageTagType>();
    let frag_data_size = whole_size / frag_count;
    let frag_full_size = size_of::<NetworkMessageFragmentTail>() + frag_data_size;

    let mut fragments = Vec::with_capacity(1 + frag_count);
    let frag_id_slice = &fragment_id.to_be_bytes();

    for i in 0..frag_count {
        let mut data = Vec::with_capacity(frag_full_size);
        data.extend_from_slice(&whole_bytes[(i * frag_data_size)..((i + 1) * frag_data_size)]);

        push_tail_data(&mut data, frag_id_slice, i as u8, 1 + frag_count as u8);
        fragments.push(data);
    }

    let mut data =
        Vec::with_capacity(size_of::<NetworkMessageFragmentTail>() + (whole_size % frag_count));
    data.extend_from_slice(&whole_bytes[(frag_count * frag_data_size)..]);
    data.extend(inner_tag.to_be_bytes());

    push_tail_data(
        &mut data,
        frag_id_slice,
        frag_count as u8,
        1 + frag_count as u8,
    );
    fragments.push(data);

    fragments
}

fn push_tail_data(data: &mut Vec<u8>, id_slice: &[u8], index: u8, total: u8) {
    data.extend_from_slice(id_slice);
    data.push(index);
    data.push(total);
    data.push(0u8); // padding
    data.extend(FRAGMENTED_TAG_BYTES);
}
