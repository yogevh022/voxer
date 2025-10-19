use wgpu::{BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, BufferSize, ComputePipeline, ComputePipelineDescriptor, Device, PipelineLayoutDescriptor, PushConstantRange, ShaderStages};
use crate::renderer::gpu::chunk_session::GpuChunkSession;
use crate::renderer::resources;
use crate::renderer::resources::vx_buffer::VxBuffer;

pub (crate) struct GpuChunkSessionResources;

impl GpuChunkSessionResources {
    pub (crate) fn chunk_render_bind_group(
        device: &Device,
        camera_buffer: &VxBuffer,
        face_data_buffer: &VxBuffer,
    ) -> (BindGroupLayout, BindGroup) {
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Chunk Render Bind Group Layout"),
            entries: &[
                camera_buffer.bind_layout_entry(0, false, ShaderStages::VERTEX),
                face_data_buffer.bind_layout_entry(1, true, ShaderStages::VERTEX),
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Chunk Render Bind Group"),
            layout: &layout,
            entries: &resources::utils::bind_entries([
                camera_buffer.as_entire_binding(),
                face_data_buffer.as_entire_binding(),
            ]),
        });
        (layout, bind_group)
    }

    pub (crate) fn chunk_meshing_bgl(
        device: &Device,
        chunk_buffer_size: BufferSize,
        face_data_buffer_size: BufferSize,
        mesh_queue_buffer_size: BufferSize,
    ) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Chunk Compute Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0, // chunk data
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(chunk_buffer_size),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1, // face data
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: Some(face_data_buffer_size),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2, // mesh queue
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(mesh_queue_buffer_size),
                    },
                    count: None,
                },
            ],
        })
    }

    pub (crate) fn chunk_write_bgl(device: &Device, dst_size: BufferSize, src_size: BufferSize) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Chunk Write Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0, // write src
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(src_size),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1, // write dst
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: Some(dst_size),
                    },
                    count: None,
                },
            ],
        })
    }

    pub (crate) fn mdi_args_bgl(
        device: &Device,
        indirect_size: BufferSize,
        packed_indirect_size: BufferSize,
        meshing_batch_size: BufferSize,
        chunks_size: BufferSize,
        chunks_in_view_size: BufferSize,
        camera_size: BufferSize,
    ) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Chunk Draw Args Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0, // indirect
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: Some(indirect_size),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1, // meshing indirect
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: Some(packed_indirect_size),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2, // meshing batch
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: Some(meshing_batch_size),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3, // chunks
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(chunks_size),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4, // chunks in view
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(chunks_in_view_size),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 5, // camera
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(camera_size),
                    },
                    count: None,
                },
            ],
        })
    }

    pub (crate) fn mdi_args_pipeline(device: &Device, bind_group_layouts: &[&BindGroupLayout]) -> ComputePipeline {
        let shader = resources::shader::create_shader(
            device,
            resources::shader::chunk_mdi_args_wgsl().into(),
            "Chunk Culled MDI Args Pipeline Shader",
        );
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Chunk Culled MDI Args Pipeline Layout"),
            bind_group_layouts,
            push_constant_ranges: &[PushConstantRange {
                stages: ShaderStages::COMPUTE,
                range: 0..4,
            }],
        });

        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Chunk Culled MDI Args Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("write_culled_mdi"),
            compilation_options: Default::default(),
            cache: None,
        })
    }

    pub (crate) fn chunk_meshing_pipeline(
        device: &Device,
        bind_group_layouts: &[&BindGroupLayout],
    ) -> ComputePipeline {
        let shader = resources::shader::create_shader(
            device,
            resources::shader::chunk_meshing_wgsl().into(),
            "Chunk Meshing Pipeline Shader",
        );
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Chunk Meshing Pipeline Layout"),
            bind_group_layouts,
            push_constant_ranges: &[],
        });

        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Chunk Meshing Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("mesh_chunks_entry"),
            compilation_options: Default::default(),
            cache: None,
        })
    }

    pub (crate) fn chunk_write_pipeline(
        device: &Device,
        bind_group_layouts: &[&BindGroupLayout],
    ) -> ComputePipeline {
        let shader = resources::shader::create_shader(
            device,
            resources::shader::chunk_write_wgsl().into(),
            "Chunk Write Pipeline Shader",
        );
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Chunk Write Pipeline Layout"),
            bind_group_layouts,
            push_constant_ranges: &[PushConstantRange {
                stages: ShaderStages::COMPUTE,
                range: 0..4,
            }],
        });

        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Chunk Write Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("chunk_write_entry"),
            compilation_options: Default::default(),
            cache: None,
        })
    }
}