use bytemuck::{Pod, Zeroable};
use engine_assets::{face_from_normal, CubeFace, UvRect};
use glam::Vec3;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Debug, Default, Clone)]
pub struct SolidMesh {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
}

pub fn face_uvs(face: CubeFace, uv: UvRect) -> [[f32; 2]; 4] {
    let [u0, v0] = uv.min;
    let [u1, v1] = uv.max;
    match face {
        CubeFace::Right => [
            [u0, v1],
            [u0, v0],
            [u1, v0],
            [u1, v1],
        ],
        CubeFace::Left => [
            [u0, v1],
            [u1, v1],
            [u1, v0],
            [u0, v0],
        ],
        CubeFace::Front | CubeFace::Back | CubeFace::Top => [
            [u0, v1],
            [u1, v1],
            [u1, v0],
            [u0, v0],
        ],
        CubeFace::Bottom => [
            [u1, v1],
            [u1, v0],
            [u0, v0],
            [u0, v1],
        ],
    }
}

pub fn face_corners(origin: Vec3, normal: [f32; 3]) -> [Vec3; 4] {
    let [nx, ny, nz] = normal;

    if nx > 0.0 {
        [
            origin + Vec3::new(1.0, 0.0, 0.0),
            origin + Vec3::new(1.0, 1.0, 0.0),
            origin + Vec3::new(1.0, 1.0, 1.0),
            origin + Vec3::new(1.0, 0.0, 1.0),
        ]
    } else if nx < 0.0 {
        [
            origin + Vec3::new(0.0, 0.0, 1.0),
            origin + Vec3::new(0.0, 1.0, 1.0),
            origin + Vec3::new(0.0, 1.0, 0.0),
            origin + Vec3::new(0.0, 0.0, 0.0),
        ]
    } else if ny > 0.0 {
        [
            origin + Vec3::new(0.0, 1.0, 0.0),
            origin + Vec3::new(1.0, 1.0, 0.0),
            origin + Vec3::new(1.0, 1.0, 1.0),
            origin + Vec3::new(0.0, 1.0, 1.0),
        ]
    } else if ny < 0.0 {
        [
            origin + Vec3::new(0.0, 0.0, 1.0),
            origin + Vec3::new(1.0, 0.0, 1.0),
            origin + Vec3::new(1.0, 0.0, 0.0),
            origin + Vec3::new(0.0, 0.0, 0.0),
        ]
    } else if nz > 0.0 {
        [
            origin + Vec3::new(0.0, 0.0, 1.0),
            origin + Vec3::new(1.0, 0.0, 1.0),
            origin + Vec3::new(1.0, 1.0, 1.0),
            origin + Vec3::new(0.0, 1.0, 1.0),
        ]
    } else {
        [
            origin + Vec3::new(1.0, 0.0, 0.0),
            origin + Vec3::new(1.0, 1.0, 0.0),
            origin + Vec3::new(0.0, 1.0, 0.0),
            origin + Vec3::new(0.0, 0.0, 0.0),
        ]
    }
}

pub fn append_face(mesh: &mut SolidMesh, origin: Vec3, normal: [f32; 3], uv: UvRect) {
    let face = face_from_normal(normal);
    let corners = face_corners(origin, normal);
    let uvs = face_uvs(face, uv);
    let base = mesh.vertices.len() as u32;

    for (corner, tile_uv) in corners.iter().zip(uvs) {
        mesh.vertices.push(MeshVertex {
            position: corner.to_array(),
            normal,
            uv: tile_uv,
        });
    }

    mesh.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}
