struct PostUniform {
    fog_color: vec4<f32>,
    fog_density: f32,
    near: f32,
    far: f32,
    _align_pad: f32,
    _pad: vec4<f32>,
};

@group(0) @binding(0)
var hdr_tex: texture_2d<f32>;
@group(0) @binding(1)
var depth_tex: texture_depth_2d;
@group(0) @binding(2)
var post_sampler: sampler;
@group(0) @binding(3)
var<uniform> post: PostUniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var out: VertexOutput;
    let pos = positions[vertex_index];
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = pos * 0.5 + 0.5;
    return out;
}

fn aces_tonemap(color: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((color * (a * color + b)) / (color * (c * color + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn linearize_depth(d: f32) -> f32 {
    let z = d * 2.0 - 1.0;
    return (2.0 * post.near * post.far) / (post.far + post.near - z * (post.far - post.near));
}

fn ssao_factor(uv: vec2<f32>, depth: f32) -> f32 {
    let texel = 1.0 / vec2<f32>(textureDimensions(depth_tex));
    var occlusion = 0.0;
    let center = linearize_depth(depth);
    let samples = array<vec2<f32>, 4>(
        vec2<f32>(1.0, 0.0),
        vec2<f32>(-1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, -1.0),
    );
    for (var i = 0; i < 4; i++) {
        let offset_uv = uv + samples[i] * texel * 2.5;
        let sample_dims = textureDimensions(depth_tex);
        let sample_coord = vec2<i32>(clamp(offset_uv * vec2<f32>(sample_dims), vec2<f32>(0.0), vec2<f32>(sample_dims) - 1.0));
        let sample_depth = textureLoad(depth_tex, sample_coord, 0);
        let linear = linearize_depth(sample_depth);
        occlusion += select(1.0, 0.0, center - linear < 0.4);
    }
    return 1.0 - occlusion * 0.18;
}

@fragment
fn fs_composite(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = vec2<f32>(input.uv.x, 1.0 - input.uv.y);
    var color = textureSample(hdr_tex, post_sampler, uv).rgb;
    let depth_dims = textureDimensions(depth_tex);
    let depth_coord = vec2<i32>(clamp(uv * vec2<f32>(depth_dims), vec2<f32>(0.0), vec2<f32>(depth_dims) - 1.0));
    let depth = textureLoad(depth_tex, depth_coord, 0);

    if depth >= 0.9999 {
        return vec4<f32>(aces_tonemap(color), 1.0);
    }

    let ao = ssao_factor(uv, depth);
    color *= ao;

    let fog = 1.0 - exp(-post.fog_density * linearize_depth(depth));
    color = mix(color, post.fog_color.rgb, fog);

    color = aces_tonemap(color);
    return vec4<f32>(color, 1.0);
}
