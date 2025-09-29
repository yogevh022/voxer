use std::net::SocketAddr;
use std::sync::atomic::AtomicU32;
use rustc_hash::FxHashMap;
use std::time::{Duration, Instant};
use crate::types::{MessageBytes, MsgFrag, MsgFragTail, NetworkMessageTag};

static FRAGMENT_ID_COUNTER: AtomicU32 = AtomicU32::new(0);
pub(crate) const FRAGMENTED_MSG_TAG: NetworkMessageTag = NetworkMessageTag::MAX;
pub(crate) const FRAGMENTED_MSG_TAG_BYTES: [u8; size_of::<NetworkMessageTag>()] = FRAGMENTED_MSG_TAG.to_be_bytes();

#[derive(Debug)]
struct MsgFragEntry {
    last_updated: Instant,
    frags: Vec<MsgFrag>,
}

impl MsgFragEntry {
    pub fn push_frag(&mut self, frag: MsgFrag) {
        self.frags.push(frag);
        self.last_updated = Instant::now();
    }

    pub fn ready(&self) -> bool {
        self.frags.len() == self.frags.first().unwrap().tail.total as usize
    }
}

impl Default for MsgFragEntry {
    fn default() -> Self {
        Self {
            last_updated: Instant::now(),
            frags: Vec::new(),
        }
    }
}

pub struct MsgFragAssembler {
    gc_timer: Instant,
    gc_interval: Duration,
    fragment_timeout: Duration,
    fragments: FxHashMap<(SocketAddr, u32), MsgFragEntry>,
}

impl MsgFragAssembler {
    pub fn insert_fragment(&mut self, src: SocketAddr, fragment: MsgFrag) -> Option<MessageBytes> {
        let frag_id = fragment.tail.id;
        let fragments_entry = self.fragments.entry((src, frag_id)).or_default();
        fragments_entry.push_frag(fragment);
        if fragments_entry.ready() {
            let ready_frags = self.fragments.remove(&(src, frag_id)).unwrap();
            return Some(MsgFragAssembler::assemble_fragments(ready_frags.frags));
        }
        None
    }

    pub fn gc_pass(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.gc_timer) > self.gc_interval {
            self.fragments
                .retain(|_, v| v.last_updated.elapsed() < self.fragment_timeout);
            self.gc_timer = now;
        }
    }

    fn assemble_fragments(mut fragments: Vec<MsgFrag>) -> MessageBytes {
        fragments.sort_by(|a, b| a.tail.index.cmp(&b.tail.index));
        let mut data = Vec::with_capacity(fragments.iter().map(|f| f.data.len()).sum());
        for fragment in fragments {
            data.extend_from_slice(&fragment.data);
        }
        data
    }
}

impl Default for MsgFragAssembler {
    fn default() -> Self {
        Self {
            gc_timer: Instant::now(),
            gc_interval: Duration::from_millis(200),
            fragment_timeout: Duration::from_millis(1000),
            fragments: FxHashMap::default(),
        }
    }
}


pub(crate) fn fragment_bytes(
    whole_bytes: &[u8],
    frag_count: usize,
    inner_tag: NetworkMessageTag,
) -> Vec<MessageBytes> {
    let fragment_group_id: u32 = FRAGMENT_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let whole_size = whole_bytes.len() + size_of::<NetworkMessageTag>();
    let frag_data_size = whole_size / frag_count;
    let frag_full_size = size_of::<MsgFragTail>() + frag_data_size;

    let mut fragments = Vec::with_capacity(1 + frag_count);
    let frag_id_slice = &fragment_group_id.to_be_bytes();

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
    data.extend(FRAGMENTED_MSG_TAG_BYTES);
}
