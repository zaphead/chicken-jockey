struct VoxelBuffer {
    data: array<u32>,
}

struct CounterBuffer {
    count: atomic<u32>,
}

@group(0) @binding(0) var<storage, read> voxels: VoxelBuffer;
@group(0) @binding(1) var<storage, read_write> counter: CounterBuffer;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= arrayLength(&voxels.data) {
        return;
    }
    if voxels.data[gid.x] != 0u {
        atomicAdd(&counter.count, 1u);
    }
}
