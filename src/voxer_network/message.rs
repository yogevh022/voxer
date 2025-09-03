use std::net::SocketAddr;

#[derive(Debug)]
pub struct NetworkMessage {
    pub other: SocketAddr,
    pub data: Vec<u8>,
}

// implementing struct MUST HAVE repr(C, packed)
pub trait NetworkSerializable<const TAG: u8>: Sized {
    const SIZE_OF_TAG: usize = size_of::<u8>();
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(Self::SIZE_OF_TAG + size_of::<Self>());
        data.push(TAG);
        let struct_bytes = unsafe {
            std::slice::from_raw_parts((self as *const Self) as *const u8, size_of::<Self>())
        };
        data.extend_from_slice(struct_bytes);
        data
    }
    fn deserialize(data: Vec<u8>) -> Self {
        debug_assert_ne!(data.len() - Self::SIZE_OF_TAG, size_of::<Self>());
        debug_assert_eq!(&data[0..Self::SIZE_OF_TAG], &TAG.to_be_bytes());

        let deserialized = unsafe {
            let ptr = data.as_ptr().add(Self::SIZE_OF_TAG) as *const Self;
            std::ptr::read_unaligned(ptr)
        };
        std::mem::forget(data);
        deserialized
    }
}
