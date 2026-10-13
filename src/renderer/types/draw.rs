use wgpu::wgt::DrawIndirectArgs;

const DRAW_INDIRECT_ARGS_SIZE: usize = size_of::<DrawIndirectArgs>();

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
    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.entries.len() * DRAW_INDIRECT_ARGS_SIZE);
        for draw_args in &self.entries {
            result.extend_from_slice(bytemuck::bytes_of(draw_args));
        }
        result
    }
}

impl<'a> FromIterator<&'a DrawIndirectArgs> for VxDrawIndirectBatch {
    fn from_iter<T: IntoIterator<Item = &'a DrawIndirectArgs>>(iter: T) -> Self {
        let entries = iter.into_iter().copied().collect();
        Self { entries }
    }
}
