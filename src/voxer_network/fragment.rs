use crate::voxer_network::message::{NetworkMessageFragment, NetworkReceiveMessage};
use rustc_hash::FxHashMap;

#[derive(Default)]
pub struct FragmentAssembler {
    fragments: FxHashMap<u32, Vec<NetworkMessageFragment>>,
}

impl FragmentAssembler {
    pub fn insert_fragment(
        &mut self,
        fragment: NetworkMessageFragment,
    ) -> Option<Vec<u8>> {
        let f_header = fragment.header;
        let fragments_entry = self.fragments.entry(fragment.header.id).or_default();
        fragments_entry.push(fragment);
        if fragments_entry.len() == f_header.total as usize {
            return Some(FragmentAssembler::assemble_fragments(
                self.fragments.remove(&f_header.id).unwrap(),
            ));
        }
        None
    }

    fn assemble_fragments(mut fragments: Vec<NetworkMessageFragment>) -> Vec<u8> {
        fragments.sort_by(|a, b| a.header.index.cmp(&b.header.index));
        let mut data = Vec::with_capacity(fragments.iter().map(|f| f.data.len()).sum());
        for fragment in fragments {
            data.extend_from_slice(&fragment.data);
        }
        data
    }
}
