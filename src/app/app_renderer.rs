use crate::app::buffer_managers::{ChunkComputeManager, ChunkRenderManager};
use crate::renderer::builder::RendererAtlas;
use crate::renderer::gpu::{
    GPUChunkEntryBuffer, GPUChunkEntryHeader, MeshVMallocMultiBuffer, MultiBufferAllocationRequest,
    MultiBufferMeshAllocation, VMallocFirstFit,
};
use crate::renderer::resources;
use crate::renderer::{Index, Renderer, RendererBuilder, Vertex};
use crate::world::types::Chunk;
use crate::{call_every, compute, vtypes};
use glam::IVec3;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::{array, mem};
use winit::window::Window;

const VOID_MESH_OFFSET: usize = 8;

const STAGING_BUFF_N: usize = 2; // fixme temp number

pub struct AppRenderer<'window, const BUFF_N: usize> {
    pub renderer: Renderer<'window>,

    pub chunk_render: ChunkRenderManager<BUFF_N>,
    pub chunk_compute: ChunkComputeManager<STAGING_BUFF_N>,

    pub staged_chunks: HashMap<IVec3, GPUChunkEntryHeader>,
    pub loaded_chunks: Arc<RwLock<HashMap<IVec3, GPUChunkEntryHeader>>>,
    pub remove_queue: HashSet<IVec3>,

    pub chunk_malloc: MeshVMallocMultiBuffer<VMallocFirstFit, BUFF_N>,
    pub multi_buffer_allocations:
        [Vec<(usize, MultiBufferMeshAllocation<VMallocFirstFit>)>; BUFF_N],
    pub multi_buffer_remove_queue: Vec<MultiBufferMeshAllocation<VMallocFirstFit>>,

    pub transform_mats_bind_group: wgpu::BindGroup,
    pub texture_atlas: RendererAtlas,

    pub render_pipeline: wgpu::RenderPipeline,
}

impl<const BUFF_N: usize> AppRenderer<'_, BUFF_N> {
    pub fn write_new_chunks(&mut self, chunks: Vec<(usize, IVec3, Chunk)>) {
        let mut chunk_entries = GPUChunkEntryBuffer::new(chunks.len());
        for (slab_index, chunk_pos, chunk) in chunks.into_iter() {
            let face_count = compute::chunk::face_count(&chunk.blocks);
            let vertex_count = face_count * 4;
            let index_count = face_count * 6;
            let alloc_request = MultiBufferAllocationRequest {
                vertex_size: vertex_count,
                index_size: index_count,
            };
            let mb_mesh_malloc = self.chunk_malloc.alloc(alloc_request).unwrap();
            let header = GPUChunkEntryHeader::new(
                mb_mesh_malloc.vertex_offset as u32,
                mb_mesh_malloc.index_offset as u32,
                vertex_count as u32,
                index_count as u32,
                slab_index as u32,
                chunk_pos,
            );
            self.multi_buffer_allocations[mb_mesh_malloc.buffer_index]
                .push((slab_index, mb_mesh_malloc));
            self.staged_chunks.insert(chunk_pos, header.clone());
            chunk_entries.insert(header, chunk.blocks);
        }

        self.renderer.write_buffer(
            &self.staging_chunk_buff,
            0,
            &bytemuck::cast_slice(&chunk_entries),
        );

        // self.chunk_malloc.vertex.draw_cli();
    }

    pub fn unload_chunks(&mut self, chunks: Vec<IVec3>) {
        self.remove_queue.extend(chunks);
        let mut loaded_chunks = self.loaded_chunks.write().unwrap();
        for mesh_allocation in self.multi_buffer_remove_queue.drain(..) {
            self.chunk_malloc.free(mesh_allocation).unwrap();
        }
        // self.remove_queue.retain(|c_pos| {
        //     loaded_chunks
        //         .remove(c_pos)
        //         .and_then(|chunk_entry| {
        //             let req = MultiBufferAllocationRequest {
        //                 vertex_size: chunk_entry.vertex_count as usize,
        //                 index_size: chunk_entry.index_count as usize,
        //             };
        //             self.chunk_malloc.free(req);
        //             Some(false)
        //         })
        //         .unwrap_or(true)
        // });

        // self.gpu_vertex_malloc.draw_cli();
    }

    pub fn compute_chunks(&mut self) {
        let mut encoder = create_encoder(&self.renderer.device);
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute_pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.chunk_compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(self.staged_chunks.len() as u32, 1, 1);
        }
        for chunk in self.staged_chunks.values() {
            let vertex_offset_bytes = chunk.vertex_offset as u64 * Vertex::size() as u64;
            encoder.copy_buffer_to_buffer(
                &self.staging_vertex_buff,
                vertex_offset_bytes,
                &self.vertex_buff,
                vertex_offset_bytes,
                Some(chunk.vertex_count as u64 * Vertex::size() as u64),
            );
            let index_offset_bytes = chunk.index_offset as u64 * size_of::<Index>() as u64;
            encoder.copy_buffer_to_buffer(
                &self.staging_index_buff,
                index_offset_bytes,
                &self.index_buff,
                index_offset_bytes,
                Some(chunk.index_count as u64 * size_of::<Index>() as u64),
            );
            let mmat_offset_bytes = chunk.slab_index as u64 * size_of::<[[f32; 4]; 4]>() as u64;
            encoder.copy_buffer_to_buffer(
                &self.staging_mmat_buff,
                mmat_offset_bytes,
                &self.mmat_buff,
                mmat_offset_bytes,
                Some(size_of::<[[f32; 4]; 4]>() as u64),
            );
        }
        self.renderer.queue.submit(Some(encoder.finish()));
        let staged_chunks = mem::take(&mut self.staged_chunks);
        let loaded_chunks = self.loaded_chunks.clone();
        self.renderer.queue.on_submitted_work_done(move || {
            loaded_chunks.write().unwrap().extend(staged_chunks);
        });
    }

    pub fn encode_render_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        camera: &vtypes::Camera,
    ) {
        let mut render_pass =
            resources::render_pass::begin(encoder, view, &self.renderer.depth_texture_view);
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.texture_atlas.bind_group, &[]);

        let view_proj = camera.get_view_projection().to_cols_array();
        self.renderer.write_buffer(
            &self.renderer.view_projection_buffer,
            0u64,
            bytemuck::cast_slice(&[view_proj]),
        );

        // let loaded_chunks = self.loaded_chunks.read().unwrap();
        // if !loaded_chunks.is_empty() {
        //     let buffer_commands = loaded_chunks
        //         .values()
        //         .map(|header| header.draw_indexed_indirect_args())
        //         .collect::<Vec<_>>();
        //     drop(loaded_chunks);
        //     self.renderer.write_buffer(
        //         &self.indirect_buff,
        //         0,
        //         &bytemuck::cast_slice(&buffer_commands),
        //     );
        //     render_pass.multi_draw_indexed_indirect(
        //         &self.indirect_buff,
        //         0,
        //         buffer_commands.len() as u32,
        //     );
        // }
    }

    pub fn render(&mut self, camera: &vtypes::Camera) -> Result<(), wgpu::SurfaceError> {
        let frame = self.renderer.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = create_encoder(&self.renderer.device);
        self.encode_render_pass(&mut encoder, &view, camera);

        self.renderer.queue.submit(Some(encoder.finish()));

        frame.present();
        Ok(())
    }
}

pub fn make_app_renderer<'a, const BUFF_N: usize>(
    window: Arc<Window>,
    render_distance: f32,
) -> AppRenderer<'a, BUFF_N> {
    let renderer_builder = RendererBuilder::new(window);
    let texture_atlas = renderer_builder.make_atlas(get_atlas_image());

    // >upper bound of max chunks to be buffered at once
    let max_rendered_chunks = compute::geo::max_discrete_sphere_pts(render_distance);
    let temp_size = (compute::MIB * 128) as u64;

    let (staging_chunk_buff, staging_chunk_buff_layout) =
        chunk_blocks_buffer(&renderer_builder, temp_size as usize);
    let mmat_buffer = RendererBuilder::make_buffer(
        &renderer_builder.device.as_ref().unwrap(),
        "mmat_buffer",
        (max_rendered_chunks * size_of::<[[f32; 4]; 4]>()) as u64,
        wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
    );

    let surface_format = renderer_builder.surface_format.unwrap();
    let renderer = renderer_builder.build();

    let (transform_mats_bind_group_layout, transform_mats_bind_group) = transform_matrices_binds(
        &renderer.device,
        &renderer.view_projection_buffer,
        &mmat_buffer,
    );

    let chunk_compute_pipeline =
        create_chunk_compute_pipeline(&renderer.device, &[&staging_chunk_buff_layout]);
    let render_pipeline = RendererBuilder::make_render_pipeline(
        &renderer.device,
        surface_format,
        resources::shader::main_shader().into(),
        &[
            &texture_atlas.texture_bind_group_layout,
            &transform_mats_bind_group_layout,
        ],
    );

    let chunk_render = ChunkRenderManager::<BUFF_N>::init(
        |i| {
            RendererBuilder::make_buffer(
                &renderer.device,
                &("vertex_buffer_".to_string() + &i.to_string()),
                temp_size,
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            )
        },
        |i| {
            RendererBuilder::make_buffer(
                &renderer.device,
                &("index_buffer_".to_string() + &i.to_string()),
                temp_size,
                wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            )
        },
    );

    let chunk_compute = ChunkComputeManager::<STAGING_BUFF_N>::init(
        &renderer.device,
        chunk_compute_pipeline,
        |i| {
            RendererBuilder::make_buffer(
                &renderer.device,
                &("staging_chunk_buffer_".to_string() + &i.to_string()),
                temp_size,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            )
        },
        |i| {
            RendererBuilder::make_buffer(
                &renderer.device,
                &("staging_vertex_buffer_".to_string() + &i.to_string()),
                temp_size,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            )
        },
        |i| {
            RendererBuilder::make_buffer(
                &renderer.device,
                &("staging_index_buffer_".to_string() + &i.to_string()),
                temp_size,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            )
        },
        |i| {
            RendererBuilder::make_buffer(
                &renderer.device,
                &("staging_mmat_buffer_".to_string() + &i.to_string()),
                temp_size,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            )
        },
    );

    let chunk_malloc = MeshVMallocMultiBuffer::<VMallocFirstFit, BUFF_N>::new(
        temp_size as usize / Vertex::size(),
        temp_size as usize / size_of::<Index>(),
        VOID_MESH_OFFSET,
    );

    AppRenderer {
        renderer,
        chunk_render,
        chunk_compute,
        staged_chunks: HashMap::new(),
        loaded_chunks: Arc::new(RwLock::new(HashMap::new())),
        remove_queue: HashSet::new(),
        chunk_malloc,
        multi_buffer_allocations: array::from_fn(|_| vec![]),
        multi_buffer_remove_queue: vec![],
        transform_mats_bind_group,
        texture_atlas,
        render_pipeline,
    }
}

pub fn transform_matrices_binds(
    device: &wgpu::Device,
    view_proj_buffer: &wgpu::Buffer,
    model_mat_buffer: &wgpu::Buffer,
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
    let view_projection_binding = wgpu::BindingResource::Buffer(wgpu::BufferBinding {
        buffer: view_proj_buffer,
        offset: 0,
        size: std::num::NonZeroU64::new(view_proj_buffer.size()),
    });
    resources::transform_matrices::create_bind_group(
        device,
        &resources::bind_group::index_based_entries([
            view_projection_binding,              // 0
            model_mat_buffer.as_entire_binding(), // 1
        ]),
    )
}

pub fn chunk_blocks_buffer(
    renderer_builder: &RendererBuilder,
    size: usize,
) -> (wgpu::Buffer, wgpu::BindGroupLayout) {
    let block_buffer =
        resources::chunk::create_chunk_buffer(renderer_builder.device.as_ref().unwrap(), size);
    (
        block_buffer,
        resources::chunk::chunk_buffer_bind_group_layout(renderer_builder.device.as_ref().unwrap()),
    )
}

pub fn create_chunk_compute_pipeline(
    device: &wgpu::Device,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::ComputePipeline {
    let shader = resources::shader::create(
        device,
        resources::shader::chunk_meshing().into(),
    );
    resources::pipeline::create_compute(
        device,
        bind_group_layouts,
        &shader,
        "block_buffer_compute_pipeline",
    )
}

pub fn indirect_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("temp_indirect_buffer"),
        size,
        usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

pub fn get_atlas_image() -> image::RgbaImage {
    let q = std::env::current_exe().unwrap();
    let project_root = q
        .parent() // target/debug
        .unwrap()
        .parent() // target
        .unwrap()
        .parent() // project root
        .unwrap();

    let atlas_image = image::open(project_root.join("src/renderer/texture/images/atlas.png"))
        .expect("failed to load atlas.png");
    atlas_image.to_rgba8()
}

pub fn create_encoder(device: &wgpu::Device) -> wgpu::CommandEncoder {
    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("render_encoder"),
    })
}
