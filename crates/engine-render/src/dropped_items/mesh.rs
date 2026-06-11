use std::collections::HashMap;

use engine_assets::{
    face_from_normal, CubeFace, DrawCategory, ItemKind, ResolvedBlockMaterials, ResolvedFace,
    ToolRegistry, UvRect,
};
use engine_world::{BiomeMap, BlockPos};
use glam::Vec3;

use crate::camera::Camera;
use crate::extract::DroppedItemRender;
use crate::mesh::{
    face_corners, face_uvs, pack_vertex_anim, tint_index_for, MeshVertex, SolidMesh,
    VERTEX_FLAG_OVERLAY,
};
use crate::particles::{ParticleMesh, ParticleVertex};

const CUBE_SIZE: f32 = 0.25;
const BOB_SPEED: f32 = 0.875;
const BOB_AMP: f32 = 0.08;
const TOOL_SIZE: f32 = 0.28;
const MAX_STACK_VISUALS: u16 = 4;
const CLUSTER_SPREAD: f32 = 0.065;

const FACE_DATA: [([f32; 3], CubeFace); 6] = [
    ([1.0, 0.0, 0.0], CubeFace::Right),
    ([-1.0, 0.0, 0.0], CubeFace::Left),
    ([0.0, 1.0, 0.0], CubeFace::Front),
    ([0.0, -1.0, 0.0], CubeFace::Back),
    ([0.0, 0.0, 1.0], CubeFace::Top),
    ([0.0, 0.0, -1.0], CubeFace::Bottom),
];

#[derive(Debug, Default, Clone)]
pub struct ItemDropMeshes {
    pub opaque: SolidMesh,
    pub cutout: SolidMesh,
    pub tools: ParticleMesh,
}

pub struct ItemDropBuildContext<'a> {
    pub materials: &'a ResolvedBlockMaterials,
    pub biome: &'a BiomeMap,
    pub camera: &'a Camera,
    pub tools: &'a ToolRegistry,
    pub item_icons: &'a HashMap<String, UvRect>,
}

pub fn build_item_drop_meshes(
    items: &[DroppedItemRender],
    ctx: &ItemDropBuildContext<'_>,
) -> ItemDropMeshes {
    let mut meshes = ItemDropMeshes::default();
    for item in items {
        match item.kind {
            ItemKind::Block { id, state } => {
                append_block_drop(&mut meshes, item, id, state, ctx);
            }
            ItemKind::Tool(tool_id) => {
                let Some(tool) = ctx.tools.get(tool_id) else {
                    continue;
                };
                let Some(icon_uv) = ctx.item_icons.get(&tool.name).copied() else {
                    continue;
                };
                append_tool_billboard(&mut meshes.tools, ctx.camera, item, icon_uv);
            }
        }
    }
    meshes
}

fn bob_offset(spin: f32) -> f32 {
    (spin * BOB_SPEED).sin() * BOB_AMP
}

fn rotate_z(v: Vec3, cos: f32, sin: f32) -> Vec3 {
    Vec3::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos, v.z)
}

fn stack_visual_count(count: u16) -> usize {
    count.min(MAX_STACK_VISUALS).max(1) as usize
}

const CLUSTER_ONE: [Vec3; 1] = [Vec3::ZERO];
const CLUSTER_TWO: [Vec3; 2] = [
    Vec3::new(-CLUSTER_SPREAD, -0.02, 0.0),
    Vec3::new(CLUSTER_SPREAD, 0.02, 0.09),
];
const CLUSTER_THREE: [Vec3; 3] = [
    Vec3::new(-CLUSTER_SPREAD, -CLUSTER_SPREAD, 0.0),
    Vec3::new(CLUSTER_SPREAD, -CLUSTER_SPREAD, 0.08),
    Vec3::new(0.0, CLUSTER_SPREAD, 0.14),
];
const CLUSTER_FOUR: [Vec3; 4] = [
    Vec3::new(-CLUSTER_SPREAD, -CLUSTER_SPREAD, 0.0),
    Vec3::new(CLUSTER_SPREAD, -CLUSTER_SPREAD, 0.07),
    Vec3::new(-CLUSTER_SPREAD, CLUSTER_SPREAD, 0.12),
    Vec3::new(CLUSTER_SPREAD, CLUSTER_SPREAD, 0.18),
];

fn cluster_offsets(visual_count: usize) -> &'static [Vec3] {
    match visual_count {
        1 => &CLUSTER_ONE,
        2 => &CLUSTER_TWO,
        3 => &CLUSTER_THREE,
        _ => &CLUSTER_FOUR,
    }
}

fn append_block_drop(
    meshes: &mut ItemDropMeshes,
    item: &DroppedItemRender,
    block_id: engine_world::BlockId,
    state: engine_world::BlockState,
    ctx: &ItemDropBuildContext<'_>,
) {
    let cos = item.spin.cos();
    let sin = item.spin.sin();
    let bob = bob_offset(item.spin);
    let tint_pos = BlockPos::new(
        item.position.x.floor() as i32,
        item.position.y.floor() as i32,
        item.position.z.floor() as i32,
    );

    for offset in cluster_offsets(stack_visual_count(item.count)) {
        let center = item.position + rotate_z(*offset, cos, sin) + Vec3::new(0.0, 0.0, bob);
        for (normal, face) in FACE_DATA {
            let resolved = ctx.materials.resolve_face(block_id, state, face, None);
            let tint_index = tint_index_for(resolved.tint, ctx.biome, tint_pos);
            let mesh = match resolved.draw_category {
                DrawCategory::Opaque => &mut meshes.opaque,
                DrawCategory::Cutout | DrawCategory::Transparent => &mut meshes.cutout,
            };
            append_rotated_face(mesh, center, cos, sin, normal, resolved, tint_index);
        }
    }
}

fn append_rotated_face(
    mesh: &mut SolidMesh,
    center: Vec3,
    cos: f32,
    sin: f32,
    normal: [f32; 3],
    face: &ResolvedFace,
    tint_index: u32,
) {
    let cube_face = face_from_normal(normal);
    let unit_corners = face_corners(Vec3::ZERO, normal);
    let corners: [Vec3; 4] = unit_corners.map(|corner| {
        let local = (corner - Vec3::splat(0.5)) * CUBE_SIZE;
        center + rotate_z(local, cos, sin)
    });
    let rotated_normal = rotate_z(Vec3::from_array(normal), cos, sin).normalize();
    let n = rotated_normal.to_array();

    let uvs = face_uvs(cube_face, face.atlas_rect);
    let uv2_rect = face.uv2.unwrap_or(UvRect::BLACK);
    let uvs2 = face_uvs(cube_face, uv2_rect);
    let mut flags = 0u32;
    if face.has_overlay() {
        flags |= VERTEX_FLAG_OVERLAY;
    }
    let anim_packed = pack_vertex_anim(face.anim, face.atlas_rect);
    let base = mesh.vertices.len() as u32;

    for (corner, (tile_uv, tile_uv2)) in corners.iter().zip(uvs.iter().zip(uvs2.iter())) {
        mesh.vertices.push(MeshVertex {
            position: corner.to_array(),
            normal: n,
            uv: *tile_uv,
            uv2: *tile_uv2,
            tint_index,
            flags,
            anim_packed,
        });
    }

    mesh.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

fn append_tool_billboard(
    mesh: &mut ParticleMesh,
    camera: &Camera,
    item: &DroppedItemRender,
    icon_uv: UvRect,
) {
    let cos = item.spin.cos();
    let sin = item.spin.sin();
    let bob = bob_offset(item.spin);
    for offset in cluster_offsets(stack_visual_count(item.count)) {
        append_tool_billboard_at(
            mesh,
            camera,
            item.position + rotate_z(*offset, cos, sin) + Vec3::new(0.0, 0.0, bob),
            cos,
            sin,
            icon_uv,
        );
    }
}

fn append_tool_billboard_at(
    mesh: &mut ParticleMesh,
    camera: &Camera,
    center: Vec3,
    cos: f32,
    sin: f32,
    icon_uv: UvRect,
) {
    let right = camera.right() * TOOL_SIZE;
    let up = camera.up() * TOOL_SIZE;
    let rotated_right = right * cos + up * sin;
    let rotated_up = -right * sin + up * cos;
    let corners = [
        center - rotated_right - rotated_up,
        center + rotated_right - rotated_up,
        center + rotated_right + rotated_up,
        center - rotated_right + rotated_up,
    ];
    let uvs = rect_uvs(icon_uv);
    let base = mesh.vertices.len() as u16;
    for (corner, uv) in corners.iter().zip(uvs) {
        mesh.vertices.push(ParticleVertex {
            position: corner.to_array(),
            uv,
            tint_index: 0,
            alpha: 1.0,
        });
    }
    mesh.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

fn rect_uvs(rect: UvRect) -> [[f32; 2]; 4] {
    let [u0, v0] = rect.min;
    let [u1, v1] = rect.max;
    [[u0, v1], [u1, v1], [u1, v0], [u0, v0]]
}
