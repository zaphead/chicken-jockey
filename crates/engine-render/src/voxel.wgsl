struct SceneUniform {
    view_proj: mat4x4<f32>,
    animation_tick: u32,
    _align_colormap: u32,
    colormap_min: vec2<f32>,
    colormap_max: vec2<f32>,
    _struct_pad: vec2<u32>,
};

@group(0) @binding(0)
var<uniform> scene: SceneUniform;

@group(1) @binding(0)
var block_atlas: texture_2d<f32>;
@group(1) @binding(1)
var block_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) uv2: vec2<f32>,
    @location(4) tint_index: u32,
    @location(5) flags: u32,
    @location(6) anim_packed: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) uv2: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tint_index: u32,
    @location(4) flags: u32,
    @location(5) anim_packed: u32,
    @location(6) world_position: vec3<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = scene.view_proj * vec4<f32>(input.position, 1.0);
    out.uv = input.uv;
    out.uv2 = input.uv2;
    out.normal = input.normal;
    out.tint_index = input.tint_index;
    out.flags = input.flags;
    out.anim_packed = input.anim_packed;
    out.world_position = input.position;
    return out;
}

@vertex
fn vs_shadow(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = lighting.light_view_proj * vec4<f32>(input.position, 1.0);
    out.uv = input.uv;
    out.uv2 = input.uv2;
    out.normal = input.normal;
    out.tint_index = input.tint_index;
    out.flags = input.flags;
    out.anim_packed = input.anim_packed;
    out.world_position = input.position;
    return out;
}

fn animated_uv(uv: vec2<f32>, anim_packed: u32) -> vec2<f32> {
    if anim_packed == 0u {
        return uv;
    }
    let frame_count = max(anim_packed & 0xFFu, 1u);
    let frametime = max((anim_packed >> 8u) & 0xFFu, 1u);
    let stride = f32((anim_packed >> 16u) & 0xFFFFu) / 65535.0;
    let frame = (scene.animation_tick / frametime) % frame_count;
    return vec2<f32>(uv.x + f32(frame) * stride, uv.y);
}

fn sample_albedo(uv: vec2<f32>, anim_packed: u32) -> vec4<f32> {
    return textureSample(block_atlas, block_sampler, animated_uv(uv, anim_packed));
}

fn apply_tint(rgb: vec3<f32>, tint_index: u32) -> vec3<f32> {
    if tint_index == 0u {
        return rgb;
    }
    let u = scene.colormap_min.x + (scene.colormap_max.x - scene.colormap_min.x) * (f32(tint_index) / 255.0);
    let tint = textureSample(block_atlas, block_sampler, vec2<f32>(u, scene.colormap_min.y)).rgb;
    return rgb * tint;
}

@fragment
fn fs_opaque(input: VertexOutput) -> @location(0) vec4<f32> {
    let base = sample_albedo(input.uv, input.anim_packed);
    var albedo = base;
    if (input.flags & 1u) != 0u {
        var overlay = sample_albedo(input.uv2, 0u);
        overlay = vec4<f32>(apply_tint(overlay.rgb, input.tint_index), overlay.a);
        albedo = vec4<f32>(mix(base.rgb, overlay.rgb, overlay.a), 1.0);
    } else {
        albedo = vec4<f32>(apply_tint(base.rgb, input.tint_index), base.a);
    }
    return vec4<f32>(shade_lit(albedo.rgb, input.normal, input.world_position), 1.0);
}

@fragment
fn fs_cutout(input: VertexOutput) -> @location(0) vec4<f32> {
    let base = sample_albedo(input.uv, input.anim_packed);
    var albedo = base;
    if (input.flags & 1u) != 0u {
        var overlay = sample_albedo(input.uv2, 0u);
        overlay = vec4<f32>(apply_tint(overlay.rgb, input.tint_index), overlay.a);
        albedo = vec4<f32>(mix(base.rgb, overlay.rgb, overlay.a), base.a);
    } else {
        albedo = vec4<f32>(apply_tint(base.rgb, input.tint_index), base.a);
    }
    if albedo.a < 0.5 {
        discard;
    }
    return vec4<f32>(shade_lit(albedo.rgb, input.normal, input.world_position), 1.0);
}

@fragment
fn fs_depth(_input: VertexOutput) {
}
