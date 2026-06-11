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
    @location(1) uv: vec2<f32>,
    @location(2) tint_index: u32,
    @location(3) alpha: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint_index: u32,
    @location(2) alpha: f32,
};

fn transform_vertex(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = scene.view_proj * vec4<f32>(input.position, 1.0);
    out.uv = input.uv;
    out.tint_index = input.tint_index;
    out.alpha = input.alpha;
    return out;
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    return transform_vertex(input);
}

@vertex
fn vs_hdr(input: VertexInput) -> VertexOutput {
    return transform_vertex(input);
}

@fragment
fn fs_particle(input: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(block_atlas, block_sampler, input.uv);
    if albedo.a < 0.05 {
        discard;
    }
    let lit = lighting.ambient_color.rgb + lighting.sun_color.rgb * lighting.sun_strength;
    return vec4<f32>(albedo.rgb * lit, albedo.a * input.alpha);
}
