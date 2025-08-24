use crate::app::buffer_managers::{
    ChunkComputeManager, ChunkRenderManager, ComputeInstruction, WriteInstruction,
};
use crate::renderer::gpu::{
    GPUChunkEntry, GPUChunkEntryHeader, MeshVMallocMultiBuffer, MultiBufferMeshAllocation,
    MultiBufferMeshAllocationRequest, VirtualMalloc,
};
use crate::renderer::resources;
use crate::renderer::{Index, Renderer, RendererBuilder, Vertex};
use crate::world::types::Chunk;
use crate::{call_every, compute, vtypes};
use glam::{IVec3, Mat4};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::{array, mem};
use wgpu::util::DrawIndexedIndirectArgs;
use winit::window::Window;

const VOID_MESH_OFFSET: usize = 8;

const STAGING_BUFF_N: usize = 1; // fixme temp number

#[derive(Debug, Clone)]
pub struct DrawDelta<const N: usize> {
    pub changed: bool,
    pub args: [HashMap<u32, DrawIndexedIndirectArgs>; N],
}

impl<const N: usize> Default for DrawDelta<N> {
    fn default() -> Self {
        Self {
            changed: false,
            args: array::from_fn(|_| HashMap::new()),
        }
    }
}

pub struct AppRenderer<'window, const BUFF_N: usize> {
    pub renderer: Renderer<'window>,

    pub chunk_render: ChunkRenderManager<BUFF_N>,
    pub chunk_compute: ChunkComputeManager<STAGING_BUFF_N>,
    pub chunk_malloc: MeshVMallocMultiBuffer<BUFF_N>,
    pub chunk_position_to_allocation: HashMap<IVec3, (usize, MultiBufferMeshAllocation)>,

    pub current_draw: [HashMap<u32, DrawIndexedIndirectArgs>; BUFF_N],
    pub draw_delta: Arc<RwLock<DrawDelta<BUFF_N>>>,

    pub render_pipeline: wgpu::RenderPipeline,
}

impl<const BUFF_N: usize> AppRenderer<'_, BUFF_N> {
    pub fn write_new_chunks(&mut self, chunks: Vec<(usize, IVec3, Chunk)>) {
        let mut chunk_buffer_entries = [const { Vec::new() }; BUFF_N];
        let mut chunk_buffer_compute_instructions = [const { Vec::new() }; STAGING_BUFF_N];
        let mut draw_args_delta: [[HashMap<u32, DrawIndexedIndirectArgs>; BUFF_N]; STAGING_BUFF_N] =
            array::from_fn(|_| array::from_fn(|_| HashMap::new()));
        for (slab_index, chunk_pos, chunk) in chunks.into_iter() {
            let face_count = compute::chunk::face_count(&chunk.blocks);
            dbg!(face_count);
            let vertex_count = face_count * 4;
            let index_count = face_count * 6;
            let alloc_request = MultiBufferMeshAllocationRequest {
                id: chunk.id,
                vertex_size: vertex_count,
                index_size: index_count,
            };
            let chunk_alloc = self.chunk_malloc.alloc(alloc_request).unwrap();
            let header = GPUChunkEntryHeader::new(chunk_alloc.1, slab_index as u32, chunk_pos);
            draw_args_delta[0][chunk_alloc.0].insert(
                chunk_alloc.1.vertex_offset,
                header.draw_indexed_indirect_args(),
            );
            chunk_buffer_entries[chunk_alloc.0].push(GPUChunkEntry::new(header, chunk.blocks));
            chunk_buffer_compute_instructions[0].push(ComputeInstruction {
                target_buffer: chunk_alloc.0,
                vertex_offset_bytes: (chunk_alloc.1.vertex_offset as usize * size_of::<Vertex>())
                    as u64,
                index_offset_bytes: (chunk_alloc.1.index_offset as usize * size_of::<Index>())
                    as u64,
                mmat_offset_bytes: (slab_index * size_of::<Mat4>()) as u64,
                vertex_size_bytes: (chunk_alloc.1.vertex_size as usize * size_of::<Vertex>())
                    as u64,
                index_size_bytes: (chunk_alloc.1.index_size as usize * size_of::<Index>()) as u64,
                mmat_size_bytes: size_of::<Mat4>() as u64,
            });
            self.chunk_position_to_allocation
                .insert(chunk_pos, (chunk_alloc.0, chunk_alloc.1));
        }

        let write_instructions: [WriteInstruction; BUFF_N] = array::from_fn(|i| {
            WriteInstruction {
                staging_index: 0, // fixme always zero for now
                bytes: bytemuck::cast_slice(&chunk_buffer_entries[i]),
                offset: 0,
            }
        });

        self.chunk_compute
            .write_to_staging_chunks(&self.renderer, write_instructions);

        self.compute_chunks(chunk_buffer_compute_instructions, draw_args_delta);


    }

    pub fn update_current_draw(&mut self) {
        let mut delta_lock = self.draw_delta.write();
        if delta_lock.changed {
            for i in 0..BUFF_N {
                self.current_draw[i].extend(delta_lock.args[i].drain());
            }
            delta_lock.changed = false;
        }
    }

    pub fn unload_chunks(&mut self, chunks: Vec<IVec3>) {
        for chunk_pos in chunks {
            let chunk_alloc_offset = self
                .chunk_position_to_allocation
                .remove(&chunk_pos)
                .unwrap();
            self.current_draw[chunk_alloc_offset.0].remove(&chunk_alloc_offset.1.vertex_offset);
            self.chunk_malloc.free(chunk_alloc_offset).unwrap();
        }
    }

    fn compute_chunks(
        &mut self,
        compute_instructions: [Vec<ComputeInstruction>; STAGING_BUFF_N],
        draw_args_delta: [[HashMap<u32, DrawIndexedIndirectArgs>; BUFF_N]; STAGING_BUFF_N],
    ) {
        self.chunk_compute.dispatch_staging_workgroups(
            &self.renderer,
            &self.chunk_render.mmat_buffers,
            &self.chunk_render.vertex_buffers,
            &self.chunk_render.index_buffers,
            compute_instructions,
            draw_args_delta,
            &self.draw_delta,
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

        let multi_draw_instructions = self
            .chunk_render
            .write_commands_to_indirect_buffer(&self.renderer, &self.current_draw);

        self.chunk_render
            .multi_draw(&self.renderer, &mut render_pass, multi_draw_instructions);
    }

    pub fn render(&mut self, camera: &vtypes::Camera) -> Result<(), wgpu::SurfaceError> {
        let frame = self.renderer.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.update_current_draw(); // fixme not here..

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
                (max_rendered_chunks * size_of::<Mat4>()) as u64,
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
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST, // COPY_DST needed for some reason
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
                (max_rendered_chunks * size_of::<Mat4>()) as u64,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            )
        },
    );

    let chunk_malloc = MeshVMallocMultiBuffer::new(temp_size as usize, VOID_MESH_OFFSET);

    AppRenderer {
        renderer,
        chunk_render,
        chunk_compute,
        chunk_malloc,
        chunk_position_to_allocation: HashMap::new(),
        current_draw: array::from_fn(|_| HashMap::new()),
        draw_delta: Arc::new(RwLock::new(DrawDelta::default())),
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
