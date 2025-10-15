fn unpack_mesh_face_offsets(mesh_entry: GPUChunkMeshEntry) -> array<u32, 6> {
    let px_count = mesh_entry.positive_face_count & 0x3FF;
    let py_count = (mesh_entry.positive_face_count >> 10) & 0x3FF;
    let pz_count = (mesh_entry.positive_face_count >> 20) & 0x3FF;

    let mx_count = mesh_entry.negative_face_count & 0x3FF;
    let my_count = (mesh_entry.negative_face_count >> 10) & 0x3FF;

    let px = mesh_entry.face_alloc;
    let mx = px + px_count;
    let py = mx + mx_count;
    let my = py + py_count;
    let pz = my + my_count;
    let mz = pz + pz_count;

    return array<u32, 6>(px, mx, py, my, pz, mz);
}

struct UnpackedGPUChunkMeshFaceMeta {
    offsets: array<u32, 6>,
    counts: array<u32, 6>,
}

fn unpack_mesh_face_offsets_with_counts(mesh_entry: GPUChunkMeshEntry) -> UnpackedGPUChunkMeshFaceMeta {
    let px_count = mesh_entry.positive_face_count & 0x3FF;
    let py_count = (mesh_entry.positive_face_count >> 10) & 0x3FF;
    let pz_count = (mesh_entry.positive_face_count >> 20) & 0x3FF;

    let mx_count = mesh_entry.negative_face_count & 0x3FF;
    let my_count = (mesh_entry.negative_face_count >> 10) & 0x3FF;
    let mz_count = (mesh_entry.negative_face_count >> 20) & 0x3FF;

    let px = mesh_entry.face_alloc;
    let mx = px + px_count;
    let py = mx + mx_count;
    let my = py + py_count;
    let pz = my + my_count;
    let mz = pz + pz_count;

    let offsets = array<u32, 6>(px, mx, py, my, pz, mz);
    let counts = array<u32, 6>(px_count, mx_count, py_count, my_count, pz_count, mz_count);

    return UnpackedGPUChunkMeshFaceMeta(offsets, counts);
}