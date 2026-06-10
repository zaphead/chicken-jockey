//! Writes per-block `albedo.png` files and the `cube_v1` Photoshop template.
use std::fs;
use std::path::Path;

use engine_assets::{face_region, CubeFace, PixelRect, ALBEDO_HEIGHT, ALBEDO_WIDTH};
use engine_assets::textures_asset_path;

fn main() {
    let textures_dir = textures_asset_path(env!("CARGO_MANIFEST_DIR"));
    fs::create_dir_all(textures_dir.join("blocks/grass")).expect("mkdir grass");
    fs::create_dir_all(textures_dir.join("blocks/dirt")).expect("mkdir dirt");
    fs::create_dir_all(textures_dir.join("blocks/stone")).expect("mkdir stone");
    fs::create_dir_all(textures_dir.join("layouts")).expect("mkdir layouts");

    write_albedo(&textures_dir.join("blocks/grass/albedo.png"), draw_grass_albedo);
    write_imported_dirt_albedo(&textures_dir);
    write_albedo(
        &textures_dir.join("blocks/stone/albedo.png"),
        draw_uniform_albedo(draw_stone),
    );

    let template = build_cube_v1_template();
    let template_path = textures_dir.join("layouts/cube_v1_template.png");
    template
        .save(&template_path)
        .unwrap_or_else(|error| panic!("write {}: {error}", template_path.display()));

    println!("wrote block albedos and {}", template_path.display());
}

fn write_imported_dirt_albedo(textures_dir: &Path) {
    let source_path = textures_dir.join("imports/dirt.png");
    let dest = textures_dir.join("blocks/dirt/albedo.png");
    if !source_path.exists() {
        write_albedo(&dest, draw_uniform_albedo(draw_dirt));
        return;
    }

    let source = image::open(&source_path)
        .unwrap_or_else(|error| panic!("open {}: {error}", source_path.display()))
        .into_rgba8();
    let sample = face_region(CubeFace::Front);
    let mut image = image::RgbaImage::new(ALBEDO_WIDTH, ALBEDO_HEIGHT);
    for face in [
        CubeFace::Top,
        CubeFace::Bottom,
        CubeFace::Left,
        CubeFace::Front,
        CubeFace::Right,
        CubeFace::Back,
    ] {
        copy_region(&mut image, face_region(face), &source, sample);
    }
    image
        .save(&dest)
        .unwrap_or_else(|error| panic!("write {}: {error}", dest.display()));
    println!("wrote {} from {}", dest.display(), source_path.display());
}

fn copy_region(
    dest: &mut image::RgbaImage,
    region: PixelRect,
    source: &image::RgbaImage,
    sample: PixelRect,
) {
    for py in 0..region.h {
        for px in 0..region.w {
            let pixel = *source.get_pixel(sample.x + px, sample.y + py);
            dest.put_pixel(region.x + px, region.y + py, pixel);
        }
    }
}

fn write_albedo(path: &Path, mut draw: impl FnMut(&mut image::RgbaImage)) {
    let mut image = image::RgbaImage::new(ALBEDO_WIDTH, ALBEDO_HEIGHT);
    draw(&mut image);
    image
        .save(path)
        .unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
    println!("wrote {}", path.display());
}

fn draw_grass_albedo(image: &mut image::RgbaImage) {
    for face in [
        CubeFace::Top,
        CubeFace::Bottom,
        CubeFace::Left,
        CubeFace::Front,
        CubeFace::Right,
        CubeFace::Back,
    ] {
        let region = face_region(face);
        let drawer: fn(u32, u32) -> image::Rgba<u8> = match face {
            CubeFace::Top => draw_grass_top,
            CubeFace::Bottom => draw_dirt,
            CubeFace::Left | CubeFace::Front | CubeFace::Right | CubeFace::Back => draw_grass_side,
        };
        fill_region(image, region, drawer);
    }
}

fn draw_uniform_albedo(
    drawer: fn(u32, u32) -> image::Rgba<u8>,
) -> impl FnMut(&mut image::RgbaImage) {
    move |image: &mut image::RgbaImage| {
        for face in [
            CubeFace::Top,
            CubeFace::Bottom,
            CubeFace::Left,
            CubeFace::Front,
            CubeFace::Right,
            CubeFace::Back,
        ] {
            fill_region(image, face_region(face), drawer);
        }
    }
}

fn fill_region(
    image: &mut image::RgbaImage,
    region: PixelRect,
    mut drawer: impl FnMut(u32, u32) -> image::Rgba<u8>,
) {
    for py in 0..region.h {
        for px in 0..region.w {
            image.put_pixel(region.x + px, region.y + py, drawer(px, py));
        }
    }
}

fn build_cube_v1_template() -> image::RgbaImage {
    let mut image = image::RgbaImage::new(ALBEDO_WIDTH, ALBEDO_HEIGHT);
    for y in 0..ALBEDO_HEIGHT {
        for x in 0..ALBEDO_WIDTH {
            image.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
        }
    }

    let faces: [(CubeFace, image::Rgba<u8>, image::Rgba<u8>, char); 6] = [
        (CubeFace::Top, image::Rgba([56, 180, 56, 255]), image::Rgba([255, 255, 255, 255]), 'T'),
        (
            CubeFace::Bottom,
            image::Rgba([255, 160, 32, 255]),
            image::Rgba([0, 0, 0, 255]),
            'D',
        ),
        (CubeFace::Left, image::Rgba([220, 48, 48, 255]), image::Rgba([255, 255, 255, 255]), 'L'),
        (
            CubeFace::Front,
            image::Rgba([48, 96, 220, 255]),
            image::Rgba([255, 255, 255, 255]),
            'F',
        ),
        (
            CubeFace::Right,
            image::Rgba([240, 220, 48, 255]),
            image::Rgba([0, 0, 0, 255]),
            'R',
        ),
        (
            CubeFace::Back,
            image::Rgba([160, 64, 200, 255]),
            image::Rgba([255, 255, 255, 255]),
            'A',
        ),
    ];

    for (face, fill, ink, label) in faces {
        let region = face_region(face);
        fill_region(&mut image, region, |_, _| fill);
        draw_face_label(&mut image, region, label, ink);
    }
    image
}

/// 5×7 pixel glyph, centered and scaled 2× inside a 16×16 face.
fn draw_face_label(
    image: &mut image::RgbaImage,
    region: PixelRect,
    ch: char,
    ink: image::Rgba<u8>,
) {
    let Some(glyph) = glyph_5x7(ch) else {
        return;
    };
    let scale = 2u32;
    let glyph_w = 5 * scale;
    let glyph_h = 7 * scale;
    let origin_x = region.x + (region.w.saturating_sub(glyph_w)) / 2;
    let origin_y = region.y + (region.h.saturating_sub(glyph_h)) / 2;
    for (row, line) in glyph.iter().enumerate() {
        for col in 0..5 {
            if line & (1 << (4 - col)) == 0 {
                continue;
            }
            for sy in 0..scale {
                for sx in 0..scale {
                    image.put_pixel(
                        origin_x + col * scale + sx,
                        origin_y + row as u32 * scale + sy,
                        ink,
                    );
                }
            }
        }
    }
}

fn glyph_5x7(ch: char) -> Option<[u8; 7]> {
    Some(match ch {
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000, 0b10000],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        _ => return None,
    })
}

fn draw_grass_top(px: u32, py: u32) -> image::Rgba<u8> {
    let noise = ((px * 3 + py * 5) % 7) as i32 - 3;
    let g = (118 + noise).clamp(80, 150) as u8;
    image::Rgba([55, g, 45, 255])
}

fn draw_grass_side(px: u32, py: u32) -> image::Rgba<u8> {
    if py < 4 {
        return draw_grass_top(px, py);
    }
    draw_dirt(px, py)
}

fn draw_dirt(px: u32, py: u32) -> image::Rgba<u8> {
    let noise = ((px * 7 + py * 11) % 9) as i32 - 4;
    let r = (115 + noise).clamp(70, 150) as u8;
    let g = (82 + noise / 2).clamp(50, 110) as u8;
    let b = (48 + noise / 3).clamp(30, 80) as u8;
    image::Rgba([r, g, b, 255])
}

fn draw_stone(px: u32, py: u32) -> image::Rgba<u8> {
    let noise = ((px * 5 + py * 3) % 11) as i32 - 5;
    let v = (140 + noise).clamp(90, 180) as u8;
    image::Rgba([v, v, (v as i32 + 4).clamp(90, 185) as u8, 255])
}
