//fn mesh_entry_face_counts(mesh_entry: GPUChunkMeshEntry) -> array<u32, 6> {
//    let px_count = mesh_entry.positive_faces & 0x3FF;
//    let py_count = (mesh_entry.positive_faces >> 10) & 0x3FF;
//    let pz_count = (mesh_entry.positive_faces >> 20) & 0x3FF;
//
//    let mx_count = mesh_entry.negative_faces & 0x3FF;
//    let my_count = (mesh_entry.negative_faces >> 10) & 0x3FF;
//    let mz_count = (mesh_entry.negative_faces >> 20) & 0x3FF;
//
//    return array<u32, 6>(px_count, mx_count, py_count, my_count, pz_count, mz_count);
//}

// fixme where does this belong?
fn mesh_face_counts(faces_p: vec3<u32>, faces_m: vec3<u32>) -> array<u32, 6> {
    return array<u32, 6>(faces_p.x, faces_m.x, faces_p.y, faces_m.y, faces_p.z, faces_m.z);
}

struct GPUChunkMeshEntryNoMeshing {
    flag: u32,
    entry: GPUChunkMeshEntry
}

fn mesh_entry_consume_meshing_flag(mesh_entry: GPUChunkMeshEntry) -> GPUChunkMeshEntryNoMeshing {
    var out: GPUChunkMeshEntryNoMeshing;
    out.entry = mesh_entry;
    out.flag = out.entry.face_alloc >> 31;
    out.entry.face_alloc = out.entry.face_alloc & 0x7FFFFFFF;
    return out;
}

fn mesh_entry_face_offsets(offset: u32, counts: array<u32, 6>) -> array<u32, 6> {
    let px_offset = offset;
    let mx_offset = px_offset + counts[0];
    let py_offset = mx_offset + counts[1];
    let my_offset = py_offset + counts[2];
    let pz_offset = my_offset + counts[3];
    let mz_offset = pz_offset + counts[4];

    return array<u32, 6>(px_offset, mx_offset, py_offset, my_offset, pz_offset, mz_offset);
}
