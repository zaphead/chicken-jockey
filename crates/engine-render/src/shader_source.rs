use std::borrow::Cow;
use std::sync::OnceLock;

static VOXEL_SHADER: OnceLock<String> = OnceLock::new();
static MINING_OVERLAY_SHADER: OnceLock<String> = OnceLock::new();

fn concat_wgsl(main: &str) -> String {
    format!("{}{}", include_str!("lighting_shared.wgsl"), main)
}

pub fn voxel_shader_source() -> wgpu::ShaderSource<'static> {
    let source = VOXEL_SHADER.get_or_init(|| concat_wgsl(include_str!("voxel.wgsl")));
    wgpu::ShaderSource::Wgsl(Cow::Borrowed(source.as_str()))
}

pub fn mining_overlay_shader_source() -> wgpu::ShaderSource<'static> {
    let source =
        MINING_OVERLAY_SHADER.get_or_init(|| concat_wgsl(include_str!("mining_overlay.wgsl")));
    wgpu::ShaderSource::Wgsl(Cow::Borrowed(source.as_str()))
}
