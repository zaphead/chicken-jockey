struct SkyUniform {
    inv_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    sun_dir: vec4<f32>,
    moon_dir: vec4<f32>,
    horizon_color: vec4<f32>,
    sun_color: vec4<f32>,
    sun_strength: f32,
    moon_strength: f32,
    star_visibility: f32,
    sky_rotation_rad: f32,
    moon_phase: u32,
    moon_phase_count: u32,
    sun_rect_min: vec2<f32>,
    sun_rect_max: vec2<f32>,
    moon_rect_min: vec2<f32>,
    moon_rect_max: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> sky: SkyUniform;

@group(1) @binding(0)
var env_tex: texture_2d<f32>;
@group(1) @binding(1)
var env_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

// ~20° angular radius (4× vanilla billboard scale).
const CELESTIAL_SIN_RADIUS: f32 = 0.348;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var out: VertexOutput;
    out.position = vec4<f32>(positions[vertex_index], 1.0, 1.0);
    out.ndc = positions[vertex_index];
    return out;
}

fn hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

// Sun/moon orbit in the Y–Z plane — sky rotates around X, not Z.
fn rotate_sky_x(dir: vec3<f32>, angle: f32) -> vec3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec3<f32>(dir.x, dir.y * c - dir.z * s, dir.y * s + dir.z * c);
}

fn cube_face_uv(n: vec3<f32>) -> vec4<f32> {
    let an = abs(n);
    if an.x >= an.y && an.x >= an.z {
        return vec4(-n.z / n.x, n.y / n.x, select(0.0, 1.0, n.x > 0.0), 0.0);
    } else if an.y >= an.x && an.y >= an.z {
        return vec4(n.x / n.y, -n.z / n.y, select(2.0, 3.0, n.y > 0.0), 0.0);
    }
    return vec4(n.x / n.z, n.y / n.z, select(4.0, 5.0, n.z > 0.0), 0.0);
}

fn uv_to_cube_dir(face: f32, uv: vec2<f32>) -> vec3<f32> {
    if face < 0.5 {
        return normalize(vec3<f32>(1.0, uv.y, -uv.x));
    }
    if face < 1.5 {
        return normalize(vec3<f32>(-1.0, uv.y, uv.x));
    }
    if face < 2.5 {
        return normalize(vec3<f32>(uv.x, 1.0, -uv.y));
    }
    if face < 3.5 {
        return normalize(vec3<f32>(uv.x, -1.0, uv.y));
    }
    if face < 4.5 {
        return normalize(vec3<f32>(uv.x, uv.y, 1.0));
    }
    return normalize(vec3<f32>(uv.x, -uv.y, -1.0));
}

fn stars(ray: vec3<f32>, sky_rotation_rad: f32, visibility: f32) -> vec3<f32> {
    if visibility <= 0.01 {
        return vec3<f32>(0.0);
    }

    // Clip at the world ground horizon only — never a rotating celestial plane.
    let horizon_fade = smoothstep(-0.05, 0.08, ray.z);
    let fade_in = visibility * visibility * (3.0 - 2.0 * visibility);

    let celestial = normalize(rotate_sky_x(ray, -sky_rotation_rad));

    let face_data = cube_face_uv(celestial);
    let uv = face_data.xy;
    let face = face_data.z;
    let res = 56.0;
    let cell = floor(uv * res);
    let seed = vec2<f32>(face * res + cell.x, cell.y);
    let h = hash21(seed);
    if h < 0.996 {
        return vec3(0.0);
    }

    let ju = vec2<f32>(hash21(seed + 1.7), hash21(seed + 4.3)) - 0.5;
    let star_uv = (cell + 0.5 + ju * 0.32) / res;
    let star_dir = uv_to_cube_dir(face, star_uv);
    let align = dot(celestial, star_dir);
    let point = smoothstep(0.9997, 0.99992, align);
    let bright = 0.65 + 0.35 * hash21(seed + 8.1);

    return vec3<f32>(point * fade_in * horizon_fade * bright);
}

fn sample_celestial(
    ray: vec3<f32>,
    celestial_dir: vec3<f32>,
    rect_min: vec2<f32>,
    rect_max: vec2<f32>,
    phase: u32,
    phase_count: u32,
    strength: f32,
) -> vec3<f32> {
    if strength <= 0.0 {
        return vec3<f32>(0.0);
    }

    let dir = normalize(celestial_dir);
    let cos_angle = dot(normalize(ray), dir);
    let sin_angle = sqrt(max(1.0 - cos_angle * cos_angle, 0.0));
    if sin_angle > CELESTIAL_SIN_RADIUS {
        return vec3<f32>(0.0);
    }

    let up_ref = select(vec3<f32>(0.0, 0.0, 1.0), vec3<f32>(0.0, 1.0, 0.0), abs(dir.z) > 0.92);
    let tangent = normalize(cross(up_ref, dir));
    let bitangent = cross(dir, tangent);
    let offset = ray - dir * cos_angle;
    let u = dot(offset, tangent) / CELESTIAL_SIN_RADIUS * 0.5 + 0.5;
    let v = dot(offset, bitangent) / CELESTIAL_SIN_RADIUS * 0.5 + 0.5;
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 {
        return vec3<f32>(0.0);
    }

    let phase_w = 1.0 / f32(max(phase_count, 1u));
    let u0 = rect_min.x + (rect_max.x - rect_min.x) * (f32(phase) * phase_w);
    let u1 = u0 + (rect_max.x - rect_min.x) * phase_w;
    let tex_uv = vec2<f32>(
        mix(u0, u1, u),
        mix(rect_min.y, rect_max.y, v),
    );
    let texel = textureSampleLevel(env_tex, env_sampler, tex_uv, 0.0);
    let lum = max(texel.r, max(texel.g, texel.b));
    let alpha = texel.a * step(0.04, lum);
    let edge = 1.0 - smoothstep(CELESTIAL_SIN_RADIUS * 0.82, CELESTIAL_SIN_RADIUS, sin_angle);
    return texel.rgb * alpha * strength * edge * 2.4;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let clip = vec4<f32>(input.ndc, 1.0, 1.0);
    let world = sky.inv_view_proj * clip;
    let ray = normalize(world.xyz / world.w - sky.camera_pos.xyz);

    let sun_dir = normalize(sky.sun_dir.xyz);
    let sun_height = sun_dir.z;
    let elevation = ray.z * 0.5 + 0.5;
    let sun_dot = dot(ray, sun_dir);

    let day_zenith = vec3<f32>(0.22, 0.52, 0.92);
    let night_zenith = vec3<f32>(0.045, 0.075, 0.17);
    let zenith = mix(night_zenith, day_zenith, sky.sun_strength);

    let horizon = sky.horizon_color.rgb;
    let elevation_power = mix(0.72, 0.38, sky.star_visibility);
    var sky_rgb = mix(horizon, zenith, pow(clamp(elevation, 0.0, 1.0), elevation_power));

    let golden_hour = (1.0 - smoothstep(0.0, 0.18, abs(sun_height)))
        * smoothstep(-0.15, 0.12, sun_height);
    let near_sun = pow(max(sun_dot, 0.0), 48.0);
    let sunset_glow = vec3<f32>(1.0, 0.45, 0.15) * near_sun * golden_hour * 0.85;
    sky_rgb += sunset_glow;

    let rim = smoothstep(0.08, 0.35, elevation) * (1.0 - smoothstep(0.35, 0.75, elevation));
    sky_rgb += horizon * rim * golden_hour * 0.25;

    sky_rgb *= mix(1.0, 0.34, sky.star_visibility);

    sky_rgb += stars(ray, sky.sky_rotation_rad, sky.star_visibility) * vec3<f32>(0.92, 0.95, 1.0);

    sky_rgb += sample_celestial(
        ray,
        sun_dir,
        sky.sun_rect_min,
        sky.sun_rect_max,
        0u,
        1u,
        sky.sun_strength,
    );

    sky_rgb += sample_celestial(
        ray,
        normalize(sky.moon_dir.xyz),
        sky.moon_rect_min,
        sky.moon_rect_max,
        sky.moon_phase,
        sky.moon_phase_count,
        sky.moon_strength,
    );

    return vec4<f32>(sky_rgb, 1.0);
}
