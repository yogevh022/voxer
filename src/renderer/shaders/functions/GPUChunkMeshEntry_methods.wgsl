
fn mesh_entry_face_counts(faces_p: vec3<u32>, faces_m: vec3<u32>) -> array<u32, 6> {
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
