use crate::app::buffer_managers::{ChunkComputeManager, ChunkRenderManager, ComputeInstruction};
use crate::renderer::builder::RendererAtlas;
use crate::renderer::gpu::{
    GPUChunkEntryBuffer, GPUChunkEntryHeader, MeshVMallocMultiBuffer, MultiBufferAllocationRequest,
    MultiBufferMeshAllocation, VMallocFirstFit, VirtualMalloc,
};
use crate::renderer::resources;
use crate::renderer::{Index, Renderer, RendererBuilder, Vertex};
use crate::world::types::Chunk;
use crate::{call_every, compute, vtypes};
use glam::IVec3;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::{array, mem};
use wgpu::wgt::DrawIndexedIndirectArgs;
use winit::window::Window;

const VOID_MESH_OFFSET: usize = 8;

const STAGING_BUFF_N: usize = 2; // fixme temp number

pub struct AppRenderer<'window, const BUFF_N: usize> {
    pub renderer: Renderer<'window>,

    pub chunk_render: ChunkRenderManager<BUFF_N>,
    pub chunk_compute: ChunkComputeManager<STAGING_BUFF_N>,

    pub compute_instructions: [Vec<ComputeInstruction>; BUFF_N],

    pub staged_chunks: HashMap<IVec3, GPUChunkEntryHeader>,
    pub loaded_chunks: Arc<RwLock<HashMap<IVec3, GPUChunkEntryHeader>>>,
    pub remove_queue: HashSet<IVec3>,

    pub chunk_malloc: MeshVMallocMultiBuffer<VMallocFirstFit, BUFF_N>,
    pub multi_buffer_allocations:
        [Vec<(usize, MultiBufferMeshAllocation<VMallocFirstFit>)>; BUFF_N],
    pub multi_buffer_remove_queue: Vec<MultiBufferMeshAllocation<VMallocFirstFit>>,

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
                id: chunk.id,
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

        // self.chunk_malloc.vertex.draw_cli();
    }

    pub fn unload_chunks(&mut self, chunks: Vec<IVec3>) {
        self.remove_queue.extend(chunks);
        let mut loaded_chunks = self.loaded_chunks.write().unwrap();
        for mesh_allocation in self.multi_buffer_remove_queue.drain(..) {
            self.chunk_malloc.free(mesh_allocation).unwrap();
        }
        // self.gpu_vertex_malloc.draw_cli();
    }

    pub fn compute_chunks(&mut self) {
        let mut compute_instructions = [const { Vec::new() }; BUFF_N];
        mem::swap(&mut compute_instructions, &mut self.compute_instructions);
        self.chunk_compute.dispatch_staging_workgroups(
            &self.renderer,
            &self.chunk_render.mmat_buffers,
            &self.chunk_render.vertex_buffers,
            &self.chunk_render.index_buffers,
            compute_instructions,
        );
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
        render_pass.set_bind_group(0, &self.renderer.bind_groups.texture_atlas, &[]);
        render_pass.set_bind_group(1, &self.renderer.bind_groups.view_projection, &[]);

        let view_proj = camera.get_view_projection().to_cols_array();
        self.renderer.write_buffer(
            &self.renderer.view_projection_buffer,
            0u64,
            bytemuck::cast_slice(&[view_proj]),
        );

        let buffer_commands: [Vec<DrawIndexedIndirectArgs>; BUFF_N] =
            [const { Vec::new() }; BUFF_N];
        let multi_draw_instructions = self
            .chunk_render
            .write_commands_to_indirect_buffer(&self.renderer, buffer_commands);

        self.chunk_render
            .multi_draw(&self.renderer, &mut render_pass, multi_draw_instructions);
    }

    pub fn render(&mut self, camera: &vtypes::Camera) -> Result<(), wgpu::SurfaceError> {
        let frame = self.renderer.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("render_encoder"),
                });

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

    // >upper bound of max chunks to be buffered at once
    let max_rendered_chunks = compute::geo::max_discrete_sphere_pts(render_distance);
    let temp_size = (compute::MIB * 128) as u64;

    let surface_format = renderer_builder.surface_format.unwrap();
    let renderer = renderer_builder.build();

    let chunk_render = ChunkRenderManager::<BUFF_N>::init(
        &renderer,
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
        |i| {
            RendererBuilder::make_buffer(
                &renderer.device,
                &("mmat_buffer_".to_string() + &i.to_string()),
                (max_rendered_chunks * size_of::<[[f32; 4]; 4]>()) as u64,
                wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE,
            )
        },
    );

    let render_pipeline = RendererBuilder::make_render_pipeline(
        &renderer.device,
        surface_format,
        resources::shader::main_shader().into(),
        &[
            &renderer.layouts.texture_atlas,   // 0
            &renderer.layouts.view_projection, // 1
            &renderer.layouts.mmat,            // 2
        ],
    );

    let chunk_compute = ChunkComputeManager::<STAGING_BUFF_N>::init(
        &renderer.device,
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
                (max_rendered_chunks * size_of::<[[f32; 4]; 4]>()) as u64,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            )
        },
    );

    let chunk_malloc = MeshVMallocMultiBuffer::new(temp_size as usize, VOID_MESH_OFFSET);

    AppRenderer {
        renderer,
        chunk_render,
        chunk_compute,
        compute_instructions: array::from_fn(|_| vec![]),
        staged_chunks: HashMap::new(),
        loaded_chunks: Arc::new(RwLock::new(HashMap::new())),
        remove_queue: HashSet::new(),
        chunk_malloc,
        multi_buffer_allocations: array::from_fn(|_| vec![]),
        multi_buffer_remove_queue: vec![],
        render_pipeline,
    }
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
