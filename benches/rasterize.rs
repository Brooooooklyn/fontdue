#[macro_use]
extern crate criterion;

use criterion::{measurement::WallTime, BenchmarkGroup, BenchmarkId, Criterion};
use fontdue;

type SetupFunction = fn(&mut BenchmarkGroup<WallTime>, &str, &[u8], f32);

// Scratch pad for glyphs: ⅞ g
const MESSAGE: &str = "Sphinx of black quartz, judge my vow.";
const FONTS: [(&str, &[u8]); 2] = [
    ("truetype", include_bytes!("../resources/fonts/Exo2-Regular.ttf")),
    ("opentype", include_bytes!("../resources/fonts/Exo2-Regular.otf")),
];
const SIZES: [f32; 4] = [20.0, 40.0, 60.0, 80.0];
const FUNCTIONS: [SetupFunction; 3] = [setup_rusttype, setup_ab_glyph, setup_fontdue];

fn setup(c: &mut Criterion) {
    let mut group = c.benchmark_group("rasterize");
    group.measurement_time(core::time::Duration::from_secs(4));
    for function in FUNCTIONS.iter() {
        for (label, font) in FONTS.iter() {
            for size in SIZES.iter() {
                function(&mut group, label, font, *size);
            }
        }
    }
    group.finish();
}

fn setup_rusttype(group: &mut BenchmarkGroup<WallTime>, font_label: &str, font: &[u8], size: f32) {
    use rusttype::{Font, Scale};
    let font = Font::try_from_bytes(font).unwrap();
    let parameter = format!("rusttype {} '{}' at {}", font_label, MESSAGE, size);
    group.bench_function(BenchmarkId::from_parameter(parameter), |b| {
        b.iter(|| {
            let mut len = 0;
            for character in MESSAGE.chars() {
                let glyph =
                    font.glyph(character).scaled(Scale::uniform(size)).positioned(rusttype::point(0.0, 0.0));
                let (height, width) = if let Some(rect) = glyph.pixel_bounding_box() {
                    (rect.height(), rect.width())
                } else {
                    (0, 0)
                };
                let mut bitmap = vec![0u8; (width * height) as usize];
                glyph.draw(|x, y, v| {
                    bitmap[(x as usize) + (y as usize) * width as usize] = (v * 255.0) as u8;
                });
                len += bitmap.len();
            }
            len
        });
    });
}

fn setup_ab_glyph(group: &mut BenchmarkGroup<WallTime>, font_label: &str, font: &[u8], size: f32) {
    use ab_glyph::{point, Font, FontRef, Glyph};
    let font = FontRef::try_from_slice(font).unwrap();
    let parameter = format!("ab_glyph {} '{}' at {}", font_label, MESSAGE, size);
    group.bench_function(BenchmarkId::from_parameter(parameter), |b| {
        b.iter(|| {
            let mut len = 0;
            for character in MESSAGE.chars() {
                let glyph: Glyph = font.glyph_id(character).with_scale_and_position(size, point(0.0, 0.0));
                if let Some(outlined) = font.outline_glyph(glyph) {
                    let bounds = outlined.px_bounds();
                    let width = bounds.width() as usize;
                    let height = bounds.height() as usize;
                    let length = width * height;
                    let mut bitmap = vec![0u8; length];
                    outlined.draw(|x, y, c| {
                        bitmap[(x as usize) + (y as usize) * width as usize] = (c * 255.0) as u8;
                    });
                    len += bitmap.len();
                }
            }
            len
        });
    });
}

fn setup_fontdue(group: &mut BenchmarkGroup<WallTime>, font_label: &str, font: &[u8], size: f32) {
    use fontdue::{Font, FontSettings};
    let settings = FontSettings {
        scale: size,
        ..FontSettings::default()
    };
    let font = Font::from_bytes(font, settings).unwrap();
    let parameter = format!("fontdue {} '{}' at {}", font_label, MESSAGE, size);
    group.bench_function(BenchmarkId::from_parameter(parameter), |b| {
        b.iter(|| {
            let mut len = 0;
            for character in MESSAGE.chars() {
                let (_, bitmap) = font.rasterize(character, size);
                len += bitmap.len();
            }
            len
        })
    });
}

criterion_group!(benches, setup);
criterion_main!(benches);
