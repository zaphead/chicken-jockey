struct LightingUniform {
    sun_dir: vec4<f32>,
    moon_dir: vec4<f32>,
    sun_color: vec4<f32>,
    ambient_color: vec4<f32>,
    moon_color: vec4<f32>,
    camera_pos: vec4<f32>,
    horizon_color: vec4<f32>,
    light_view_proj: mat4x4<f32>,
    sun_strength: f32,
    moon_strength: f32,
    star_visibility: f32,
    night_darkness: f32,
    specular_strength: f32,
    moon_phase: u32,
    moon_phase_count: u32,
    world_time: f32,
    fog_density: f32,
    _pad_before_rects: f32,
    sun_rect_min: vec2<f32>,
    sun_rect_max: vec2<f32>,
    moon_rect_min: vec2<f32>,
    moon_rect_max: vec2<f32>,
    sky_colormap_min: vec2<f32>,
    sky_colormap_max: vec2<f32>,
    fog_colormap_min: vec2<f32>,
    fog_colormap_max: vec2<f32>,
    _pad_end: vec2<f32>,
};

@group(2) @binding(0)
var<uniform> lighting: LightingUniform;
@group(3) @binding(0)
var shadow_map: texture_depth_2d;
@group(3) @binding(1)
var shadow_sampler: sampler_comparison;

fn shadow_factor(world_pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    let light_clip = lighting.light_view_proj * vec4<f32>(world_pos, 1.0);
    if light_clip.w <= 0.0 {
        return 1.0;
    }
    let ndc = light_clip.xyz / light_clip.w;
    if ndc.x < -1.0 || ndc.x > 1.0 || ndc.y < -1.0 || ndc.y > 1.0 {
        return 1.0;
    }
    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 1.0 - (ndc.y * 0.5 + 0.5));
    let n = normalize(normal);
    let ndotl = max(dot(n, -lighting.sun_dir.xyz), 0.0);
    let slope = sqrt(1.0 - ndotl * ndotl) / max(ndotl, 0.08);
    let bias = 0.00010 + 0.00055 * slope;
    let ref_depth = ndc.z - bias;

    var shadow = 0.0;
    let texel = 1.0 / vec2<f32>(textureDimensions(shadow_map));
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel;
            shadow += textureSampleCompareLevel(shadow_map, shadow_sampler, uv + offset, ref_depth);
        }
    }
    return shadow / 9.0;
}

fn shade_lit(rgb: vec3<f32>, normal: vec3<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let n = normalize(normal);
    let sun_dir = normalize(lighting.sun_dir.xyz);
    let moon_dir = normalize(lighting.moon_dir.xyz);
    let view_dir = normalize(lighting.camera_pos.xyz - world_pos);

    let sun_diff = max(dot(n, -sun_dir), 0.0) * lighting.sun_strength;
    let moon_diff = max(dot(n, -moon_dir), 0.0) * lighting.moon_strength;
    let ambient = lighting.ambient_color.rgb;

    let ambient_term = rgb * ambient;
    let direct = rgb * (lighting.sun_color.rgb * sun_diff + lighting.moon_color.rgb * moon_diff);
    let shadow = shadow_factor(world_pos, normal);
    let shadowed_direct = direct * mix(1.0, shadow, lighting.sun_strength);

    let half_vec = normalize(-sun_dir + view_dir);
    let spec = pow(max(dot(n, half_vec), 0.0), 32.0) * lighting.specular_strength * lighting.sun_strength;
    return ambient_term + shadowed_direct + lighting.sun_color.rgb * spec * shadow;
}
