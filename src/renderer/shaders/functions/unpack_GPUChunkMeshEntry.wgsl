fn unpack_mesh_face_counts(mesh_entry: GPUChunkMeshEntry) -> array<u32, 6> {
    let px_count = mesh_entry.positive_face_count & 0x3FF;
    let py_count = (mesh_entry.positive_face_count >> 10) & 0x3FF;
    let pz_count = (mesh_entry.positive_face_count >> 20) & 0x3FF;

    let mx_count = mesh_entry.negative_face_count & 0x3FF;
    let my_count = (mesh_entry.negative_face_count >> 10) & 0x3FF;
    let mz_count = (mesh_entry.negative_face_count >> 20) & 0x3FF;

    return array<u32, 6>(px_count, mx_count, py_count, my_count, pz_count, mz_count);
}

fn mesh_face_counts_to_offsets(counts: array<u32, 6>) -> array<u32, 6> {
    let px_offset = 0u;
    let mx_offset = counts[0];
    let py_offset = mx_offset + counts[1];
    let my_offset = py_offset + counts[2];
    let pz_offset = my_offset + counts[3];
    let mz_offset = pz_offset + counts[4];

    return array<u32, 6>(px_offset, mx_offset, py_offset, my_offset, pz_offset, mz_offset);
}
