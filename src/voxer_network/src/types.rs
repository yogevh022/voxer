use std::net::SocketAddr;
use crate::fragmentation::FRAGMENTED_MSG_TAG;
pub type NetworkMessageTag = u8;
pub type MessageBytes = Vec<u8>;

#[derive(Debug)]
pub struct ReceivedMessage {
    pub src: SocketAddr,
    pub data: MessageBytes,
}

pub enum SerializedMessage {
    Single(MessageBytes),
    Fragmented(Vec<MessageBytes>),
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
        let tag = NetworkMessageTag::from_be_bytes(
            self.data[tag_end..self.data.len()].try_into().unwrap(),
        );
        match tag {
            FRAGMENTED_MSG_TAG => {
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
