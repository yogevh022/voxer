use std::net::SocketAddr;
use super::message::{MessageBytes, MsgFrag};
use rustc_hash::FxHashMap;
use std::time::{Duration, Instant};

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
