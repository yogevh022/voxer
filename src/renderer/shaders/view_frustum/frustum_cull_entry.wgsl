
@group(1) @binding(0)
var<uniform> camera_view: UniformCameraView;
@group(1) @binding(1)
var<storage, read> face_data_buffer: array<VoxelFaceData>;