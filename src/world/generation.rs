use crate::compute::array::Array3D;
use crate::world::types::{Block, CHUNK_DIM, Chunk, ChunkBlocks};
use crossbeam::channel;
use crossbeam::channel::SendError;
use fastnoise2::generator::Generator;
use glam::IVec3;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::collections::HashSet;
use std::hash::Hash;
use std::mem::MaybeUninit;

pub type WorldGenRequest = Vec<IVec3>;
pub type WorldGenResponse = Vec<(IVec3, Chunk)>;

#[derive(Clone, Copy, Debug)]
pub struct WorldGenConfig {
    pub seed: i32,
    pub noise_scale: f64,
}

pub struct WorldGenHandle {
    config: WorldGenConfig,
    pending: HashSet<IVec3>,

    // master
    request_sender: channel::Sender<WorldGenRequest>,
    response_receiver: channel::Receiver<WorldGenResponse>,
    // slave
    response_sender: Option<channel::Sender<WorldGenResponse>>,
    request_receiver: Option<channel::Receiver<WorldGenRequest>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl WorldGenHandle {
    pub fn new(config: WorldGenConfig) -> Self {
        let (response_sender, response_receiver) = channel::unbounded::<WorldGenResponse>();
        let (request_sender, request_receiver) = channel::unbounded::<WorldGenRequest>();
        Self {
            config,
            pending: HashSet::new(),
            request_sender,
            response_receiver,
            response_sender: Some(response_sender),
            request_receiver: Some(request_receiver),
            thread: None,
        }
    }

    pub fn start_thread(&mut self) {
        if self.thread.is_some() {
            panic!("World generation thread already started");
        }
        let config = self.config.clone();
        let response_sender = self.response_sender.take().unwrap();
        let request_receiver = self.request_receiver.take().unwrap();
        self.thread = Some(std::thread::spawn(move || {
            world_generation_task(config, response_sender, request_receiver)
        }));
    }

    pub fn send(&mut self, msg: WorldGenRequest) -> Result<(), SendError<WorldGenRequest>> {
        self.pending.extend(msg.iter().cloned());
        self.request_sender.send(msg)
    }

    pub fn try_recv(&mut self) -> Result<WorldGenResponse, channel::RecvError> {
        if let Ok(response) = self.response_receiver.try_recv() {
            for (chunk_pos, _) in response.iter() {
                self.pending.remove(chunk_pos);
            }
            return Ok(response);
        }
        Err(channel::RecvError)
    }

    #[inline(always)]
    pub fn is_pending(&self, chunk_pos: &IVec3) -> bool {
        self.pending.contains(chunk_pos)
    }
}

pub fn world_generation_task(
    config: WorldGenConfig,
    send: channel::Sender<WorldGenResponse>,
    receive: channel::Receiver<WorldGenRequest>,
) {
    while let Ok(chunk_positions) = receive.recv() {
        let generated_chunks = chunk_positions
            .into_par_iter()
            .map(|chunk_pos| (chunk_pos, generate_chunk(config, chunk_pos)))
            .collect();
        send.send(generated_chunks).unwrap();
    }
}

pub(crate) fn generate_chunk(gen_config: WorldGenConfig, chunk_position: IVec3) -> Chunk {
    let (solid_count, blocks) = generate_chunk_blocks(gen_config, chunk_position);
    Chunk::new(chunk_position, blocks, solid_count)
}

fn generate_chunk_noise(
    gen_config: WorldGenConfig,
    chunk_position: IVec3,
) -> [[[f32; CHUNK_DIM]; CHUNK_DIM]; CHUNK_DIM] {
    let mut noise_uninit: [[[MaybeUninit<f32>; CHUNK_DIM]; CHUNK_DIM]; CHUNK_DIM] =
        unsafe { MaybeUninit::uninit().assume_init() };
    let noise_mut_slice: &mut [f32] = unsafe {
        std::slice::from_raw_parts_mut(
            noise_uninit.as_mut_ptr() as *mut f32,
            CHUNK_DIM * CHUNK_DIM * CHUNK_DIM,
        )
    };
    // todo reuse generators?
    let noise = fastnoise2::generator::prelude::opensimplex2().build();
    noise.gen_uniform_grid_3d(
        noise_mut_slice,
        chunk_position.x * CHUNK_DIM as i32,
        chunk_position.y * CHUNK_DIM as i32,
        chunk_position.z * CHUNK_DIM as i32,
        CHUNK_DIM as i32,
        CHUNK_DIM as i32,
        CHUNK_DIM as i32,
        gen_config.noise_scale as f32,
        gen_config.seed,
    );
    unsafe { *(noise_uninit.as_mut_ptr() as *mut [[[f32; CHUNK_DIM]; CHUNK_DIM]; CHUNK_DIM]) }
}

fn generate_chunk_blocks(
    gen_config: WorldGenConfig,
    chunk_position: IVec3,
) -> (usize, ChunkBlocks) {
    let noise = generate_chunk_noise(gen_config, chunk_position);

    let mut blocks: [[[MaybeUninit<Block>; CHUNK_DIM]; CHUNK_DIM]; CHUNK_DIM] =
        unsafe { MaybeUninit::uninit().assume_init() };
    
    let mut solid_count = 0;
    for z in 0..CHUNK_DIM {
        for y in 0..CHUNK_DIM {
            for x in 0..CHUNK_DIM {
                if noise[z][y][x] > 0.1 {
                    solid_count += 1;
                    blocks[x][y][z] = MaybeUninit::new(Block { value: 1u16 << 15 });
                } else {
                    blocks[x][y][z] = MaybeUninit::new(Block { value: 0u16 });
                }
            }
        }
    }
    let blocks3d: [[[Block; CHUNK_DIM]; CHUNK_DIM]; CHUNK_DIM] =
        unsafe { *(blocks.as_mut_ptr() as *mut [[[Block; CHUNK_DIM]; CHUNK_DIM]; CHUNK_DIM]) };

    (solid_count, Array3D(blocks3d))
}
