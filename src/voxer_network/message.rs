use bytemuck::Pod;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU32;

pub type NetworkMessageTag = u8;
pub type MessageBytes = Vec<u8>;
const FRAGMENTED_TAG: NetworkMessageTag = NetworkMessageTag::MAX;
const FRAGMENTED_TAG_BYTES: [u8; size_of::<NetworkMessageTag>()] = FRAGMENTED_TAG.to_be_bytes();

static FRAGMENT_ID_COUNTER: AtomicU32 = AtomicU32::new(0);
fn next_fragment_id() -> u32 {
    FRAGMENT_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[derive(Debug)]
pub struct ReceivedMessage {
    pub src: SocketAddr,
    pub data: MessageBytes,
}

pub enum SerializedMessage {
    Single(MessageBytes),
    Fragmented(Vec<MessageBytes>),
}

pub trait NetworkMessageConfig {
    /// A unique identifier (for each message type)
    const TAG: NetworkMessageTag;
    /// The number of fragments to split the message into, defaults to 1, can be 1..255
    const FRAGMENT_COUNT: usize = 1;
}

/// The default impl of NetworkMessage requires `repr(C)` and `Pod`
pub trait NetworkMessage: NetworkMessageConfig {
    /// size of TAG, never explicitly set this.
    const _TAG_SIZE: usize = size_of_val(&Self::TAG);
    const _ASSERT_FRAGMENT_COUNT_NON_ZERO: () = assert!(
        Self::FRAGMENT_COUNT > 0,
        "FRAGMENT_COUNT must be greater than 0"
    );
    fn serialize(&self) -> SerializedMessage;
    fn deserialize<B: AsRef<[u8]>>(data: B) -> Self;
}

impl<T> NetworkMessage for T
where
    T: Sized + Pod + NetworkMessageConfig,
{
    fn serialize(&self) -> SerializedMessage {
        let bytes = bytemuck::bytes_of(self);
        match Self::FRAGMENT_COUNT {
            1 => {
                let mut data = Vec::with_capacity(size_of::<Self>() + Self::_TAG_SIZE);
                data.extend_from_slice(bytes);
                data.extend(Self::TAG.to_be_bytes());
                SerializedMessage::Single(data)
            }
            _ => {
                let fragment_id = next_fragment_id();
                let fragmented_bytes =
                    fragment_bytes(fragment_id, bytes, Self::FRAGMENT_COUNT, Self::TAG);
                SerializedMessage::Fragmented(fragmented_bytes)
            }
        }
    }

    fn deserialize<B: AsRef<[u8]>>(data: B) -> Self {
        *bytemuck::try_from_bytes(data.as_ref()).unwrap()
    }
}

pub(crate) enum DecodedMessage {
    Single(MessageBytes),
    Fragment(MsgFrag),
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct MsgFrag {
    pub data: MessageBytes,
    pub tail: MsgFragTail,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct MsgFragTail {
    pub id: u32,
    pub index: u8,
    pub total: u8,
    _pad: u8, // make sure tag is the last byte
    pub tag: NetworkMessageTag,
}


#[derive(Debug)]
pub(crate) struct RawMsg<'a> {
    pub data: &'a [u8],
}

impl<'a> RawMsg<'a> {
    pub fn consume(self) -> DecodedMessage {
        let tag_end = self.data.len() - size_of::<NetworkMessageTag>();
        let tag =
            NetworkMessageTag::from_be_bytes(self.data[tag_end..self.data.len()].try_into().unwrap());
        match tag {
            FRAGMENTED_TAG => {
                let fragment_total = self.data[tag_end - 2];
                let fragment_index = self.data[tag_end - 3];
                let fragment_id =
                    u32::from_be_bytes(self.data[(tag_end - 7)..(tag_end - 3)].try_into().unwrap());
                let tail = MsgFragTail {
                    id: fragment_id,
                    index: fragment_index,
                    total: fragment_total,
                    _pad: 0,
                    tag,
                };
                DecodedMessage::Fragment(MsgFrag {
                    tail,
                    data: self.data[..self.data.len() - size_of::<MsgFragTail>()].to_vec(),
                })
            }
            _ => DecodedMessage::Single(self.data.to_vec()),
        }
    }
}

fn fragment_bytes(
    fragment_id: u32,
    whole_bytes: &[u8],
    frag_count: usize,
    inner_tag: NetworkMessageTag,
) -> Vec<MessageBytes> {
    let whole_size = whole_bytes.len() + size_of::<NetworkMessageTag>();
    let frag_data_size = whole_size / frag_count;
    let frag_full_size = size_of::<MsgFragTail>() + frag_data_size;

    let mut fragments = Vec::with_capacity(1 + frag_count);
    let frag_id_slice = &fragment_id.to_be_bytes();

    for i in 0..frag_count {
        let mut data = Vec::with_capacity(frag_full_size);
        data.extend_from_slice(&whole_bytes[(i * frag_data_size)..((i + 1) * frag_data_size)]);

        push_tail_data(&mut data, frag_id_slice, i as u8, 1 + frag_count as u8);
        fragments.push(data);
    }

    let mut data = Vec::with_capacity(size_of::<MsgFragTail>() + (whole_size % frag_count));
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

fn push_tail_data(data: &mut MessageBytes, id_slice: &[u8], index: u8, total: u8) {
    data.extend_from_slice(id_slice);
    data.push(index);
    data.push(total);
    data.push(0u8); // padding
    data.extend(FRAGMENTED_TAG_BYTES);
}
