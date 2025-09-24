use wgpu::wgt::{DrawIndirectArgs, Backend};

const DRAW_INDIRECT_ARGS_SIZE: usize = size_of::<DrawIndirectArgs>();
const DRAW_INDIRECT_ARGS_DX12_SIZE: usize = size_of::<DrawIndirectArgs>() + size_of::<[u32; 3]>();

pub struct VxDrawIndirectBatch {
    pub entries: Vec<DrawIndirectArgs>,
}

impl VxDrawIndirectBatch {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    #[inline]
    pub fn push(&mut self, entry: DrawIndirectArgs) {
        self.entries.push(entry);
    }

    #[inline]
    pub fn encode(&self, backend: Backend) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        match backend {
            Backend::Dx12 => self.encode_dx12(&mut result),
            _ => self.encode_vk(&mut result),
        }
        result
    }

    fn encode_vk(&self, result: &mut Vec<u8>) {
        result.reserve(self.entries.len() * DRAW_INDIRECT_ARGS_SIZE);
        for draw_args in self.entries.iter() {
            let bytes = bytemuck::bytes_of(draw_args);
            result.extend_from_slice(&bytes);
        }
    }

    fn encode_dx12(&self, result: &mut Vec<u8>) {
        result.reserve(self.entries.len() * DRAW_INDIRECT_ARGS_DX12_SIZE);
        for draw_args in self.entries.iter() {
            let bytes = &mut [0u8; DRAW_INDIRECT_ARGS_DX12_SIZE];
            bytes[0..DRAW_INDIRECT_ARGS_SIZE].copy_from_slice(bytemuck::bytes_of(draw_args));
            result.extend_from_slice(bytes);
        }
    }
}

impl<'a> FromIterator<&'a DrawIndirectArgs> for VxDrawIndirectBatch {
    fn from_iter<T: IntoIterator<Item = &'a DrawIndirectArgs>>(iter: T) -> Self {
        let entries = iter.into_iter().copied().collect();
        Self { entries }
    }
}
