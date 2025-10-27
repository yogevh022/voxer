use crossbeam::channel;
use crossbeam::channel::{Receiver, RecvError, SendError, Sender};
use glam::IVec3;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use rustc_hash::FxHashSet;
use std::thread::JoinHandle;
use crate::world::server::world::{WorldGenerator, CHUNK_DIM};
use crate::world::server::world::chunk::VoxelChunk;

pub type VoxelChunkNoise = [[[f32; CHUNK_DIM]; CHUNK_DIM]; CHUNK_DIM];

pub struct WorldGenHandle<G: WorldGenerator> {
    master_generator: G,
    pending: FxHashSet<IVec3>,

    thread: Option<JoinHandle<()>>,
    sender: Option<Sender<WorldGenRequest>>,
    receiver: Option<Receiver<WorldGenResponse>>,
}

impl<G: WorldGenerator> WorldGenHandle<G> {
    pub fn new(world_generator: G) -> Self {
        Self {
            master_generator: world_generator,
            pending: FxHashSet::default(),
            sender: None,
            receiver: None,
            thread: None,
        }
    }

    pub fn start_thread(&mut self) {
        if self.thread.is_some() {
            panic!("World generation thread already started");
        }
        let (response_sender, response_receiver) = channel::unbounded::<WorldGenResponse>();
        let (request_sender, request_receiver) = channel::unbounded::<WorldGenRequest>();
        let generator = self.master_generator.clone();

        self.sender = Some(request_sender);
        self.receiver = Some(response_receiver);
        self.thread = Some(std::thread::spawn(move || {
            world_gen_task(generator, response_sender, request_receiver)
        }));
    }

    pub fn stop_thread(&mut self) {
        if self.thread.is_none() {
            panic!("World generation thread not started");
        }
        let sender = self.sender.take().unwrap();
        sender.send(WorldGenRequest::Term).unwrap();
        let thread = self.thread.take().unwrap();
        thread.join().unwrap();
    }

    pub fn send(&mut self, msg: WorldGenRequest) -> Result<(), SendError<WorldGenRequest>> {
        if let WorldGenRequest::Chunks(positions) = &msg {
            self.pending.extend(positions.iter().cloned());
        }
        self.sender.as_ref().unwrap().send(msg)
    }

    pub fn try_recv(&mut self) -> Result<WorldGenResponse, RecvError> {
        if let Ok(response) = self.receiver.as_ref().unwrap().try_recv() {
            if let WorldGenResponse::Chunks(chunks) = &response {
                for chunk in chunks {
                    self.pending.remove(&chunk.position);
                }
            }
            return Ok(response);
        }
        Err(RecvError)
    }

    pub fn is_pending(&self, chunk_pos: &IVec3) -> bool {
        self.pending.contains(chunk_pos)
    }
}

pub enum WorldGenResponse {
    Chunks(Vec<VoxelChunk>),
    RecvError(RecvError),
    Term,
}
pub enum WorldGenRequest {
    Chunks(Vec<IVec3>),
    Term,
}

fn world_gen_task<G: WorldGenerator>(
    world_generator: G,
    sender: Sender<WorldGenResponse>,
    receiver: Receiver<WorldGenRequest>,
) {
    loop {
        let request = match receiver.recv() {
            Ok(request) => request,
            Err(e) => {
                sender.send(WorldGenResponse::RecvError(e)).unwrap();
                break;
            }
        };

        match request {
            WorldGenRequest::Chunks(positions) => {
                let chunks: Vec<VoxelChunk> = positions
                    .into_par_iter()
                    .map(|chunk_pos| world_generator.chunk(chunk_pos))
                    .collect();
                sender.send(WorldGenResponse::Chunks(chunks)).unwrap();
            }
            WorldGenRequest::Term => {
                sender.send(WorldGenResponse::Term).unwrap();
                break;
            }
        }
    }
}
