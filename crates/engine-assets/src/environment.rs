use std::path::{Path, PathBuf};

use crate::TextureAtlas;

const MOON_PHASE_FILES: [&str; 8] = [
    "moon_new_moon.png",
    "moon_waxing_crescent.png",
    "moon_first_quarter.png",
    "moon_waxing_gibbous.png",
    "moon_full_moon.png",
    "moon_waning_gibbous.png",
    "moon_third_quarter.png",
    "moon_waning_crescent.png",
];

#[derive(Debug, Clone)]
pub struct EnvironmentTextures {
    pub atlas: TextureAtlas,
    pub sun_rect: crate::UvRect,
    pub moon_strip_rect: crate::UvRect,
    pub sky_colormap_rect: crate::UvRect,
    pub fog_colormap_rect: crate::UvRect,
    pub moon_phase_count: u32,
}

pub fn environment_asset_path(manifest_dir: &str) -> PathBuf {
    crate::textures_asset_path(manifest_dir).join("environment")
}

pub fn load_environment_textures(manifest_dir: &str) -> EnvironmentTextures {
    let dir = environment_asset_path(manifest_dir);
    match pack_environment(&dir) {
        Ok(env) => env,
        Err(error) => {
            log::warn!(
                "environment atlas pack failed ({}): using fallback",
                dir.display()
            );
            log::debug!("environment pack error: {error}");
            fallback_environment()
        }
    }
}

fn load_rgba(path: &Path) -> Result<image::RgbaImage, String> {
    image::open(path)
        .map(|image| image.into_rgba8())
        .map_err(|error| format!("load {}: {error}", path.display()))
}

fn pack_environment(dir: &Path) -> Result<EnvironmentTextures, String> {
    let sun = load_rgba(&dir.join("sun.png"))?;
    let sky = load_rgba(&dir.join("sky0.png"))?;
    let fog = load_rgba(&dir.join("fog0.png"))?;

    let moon_images: Vec<_> = MOON_PHASE_FILES
        .iter()
        .map(|name| load_rgba(&dir.join(name)))
        .collect::<Result<_, _>>()?;

    let moon_w = moon_images[0].width();
    let moon_h = moon_images[0].height();
    for img in &moon_images {
        if img.width() != moon_w || img.height() != moon_h {
            return Err("moon phase size mismatch".into());
        }
    }

    let strip_w = moon_w * moon_images.len() as u32;
    let mut moon_strip = image::RgbaImage::new(strip_w, moon_h);
    for (i, img) in moon_images.iter().enumerate() {
        image::imageops::overlay(&mut moon_strip, img, i as i64 * moon_w as i64, 0);
    }

    let pad = 2u32;
    let sun_w = sun.width();
    let sun_h = sun.height();
    let sky_w = sky.width();
    let sky_h = sky.height();
    let fog_w = fog.width();
    let fog_h = fog.height();

    let width = sun_w.max(strip_w).max(sky_w).max(fog_w) + pad * 2;
    let height = sun_h + strip_h(moon_h) + sky_h + fog_h + pad * 5;
    let mut atlas = image::RgbaImage::new(width, height);

    let mut y = pad;
    let sun_x = pad;
    image::imageops::overlay(&mut atlas, &sun, sun_x as i64, y as i64);
    let sun_rect = rect_uv(sun_x, y, sun_w, sun_h, width, height);
    y += sun_h + pad;

    image::imageops::overlay(&mut atlas, &moon_strip, pad as i64, y as i64);
    let moon_strip_rect = rect_uv(pad, y, strip_w, moon_h, width, height);
    y += moon_h + pad;

    image::imageops::overlay(&mut atlas, &sky, pad as i64, y as i64);
    let sky_colormap_rect = rect_uv(pad, y, sky_w, sky_h, width, height);
    y += sky_h + pad;

    image::imageops::overlay(&mut atlas, &fog, pad as i64, y as i64);
    let fog_colormap_rect = rect_uv(pad, y, fog_w, fog_h, width, height);

    Ok(EnvironmentTextures {
        atlas: TextureAtlas {
            tile_size: 1,
            width,
            height,
            pixels: atlas.into_raw(),
        },
        sun_rect,
        moon_strip_rect,
        sky_colormap_rect,
        fog_colormap_rect,
        moon_phase_count: MOON_PHASE_FILES.len() as u32,
    })
}

fn strip_h(moon_h: u32) -> u32 {
    moon_h
}

fn rect_uv(x: u32, y: u32, w: u32, h: u32, atlas_w: u32, atlas_h: u32) -> crate::UvRect {
    let aw = atlas_w as f32;
    let ah = atlas_h as f32;
    crate::UvRect {
        min: [x as f32 / aw, y as f32 / ah],
        max: [(x + w) as f32 / aw, (y + h) as f32 / ah],
    }
}

fn fallback_environment() -> EnvironmentTextures {
    let width = 64u32;
    let height = 64u32;
    let mut pixels = vec![255u8; (width * height * 4) as usize];
    pixels[0] = 255;
    pixels[1] = 220;
    pixels[2] = 100;
    EnvironmentTextures {
        atlas: TextureAtlas {
            tile_size: 1,
            width,
            height,
            pixels,
        },
        sun_rect: crate::UvRect {
            min: [0.0, 0.0],
            max: [0.5, 0.5],
        },
        moon_strip_rect: crate::UvRect {
            min: [0.5, 0.0],
            max: [1.0, 0.5],
        },
        sky_colormap_rect: crate::UvRect {
            min: [0.0, 0.5],
            max: [1.0, 1.0],
        },
        fog_colormap_rect: crate::UvRect {
            min: [0.0, 0.5],
            max: [1.0, 1.0],
        },
        moon_phase_count: 8,
    }
}
