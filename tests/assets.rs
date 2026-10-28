use std::fs;
use std::path::{Path, PathBuf};

use mai::assets;

/// A fresh, empty fixture folder per test.
fn fixture(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mai-assets-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_png(path: &Path, width: u32, height: u32) {
    let file = fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(file, width, height);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut w = enc.write_header().unwrap();
    w.write_image_data(&vec![255; (width * height * 4) as usize])
        .unwrap();
}

#[test]
fn valid_folder_loads_idle_with_frame_count_fps_and_loop() {
    let dir = fixture("valid");
    write_png(&dir.join("idle.png"), 64 * 4, 64);
    fs::write(dir.join("character.toml"), "[idle]\nfps = 8\nloop = true\n").unwrap();

    let anims = assets::load(&dir).unwrap();
    let idle = &anims["idle"];

    assert_eq!(idle.clip.frames, 4);
    assert_eq!(idle.clip.fps, 8);
    assert!(idle.clip.looping);
    assert_eq!(idle.frame_size, 64);
}

fn load_err(dir: &Path) -> assets::AssetError {
    match assets::load(dir) {
        Ok(_) => panic!("expected an asset error"),
        Err(e) => e,
    }
}

#[test]
fn missing_png_names_the_png() {
    let dir = fixture("missing-png");
    fs::write(dir.join("character.toml"), "[idle]\nfps = 8\nloop = true\n").unwrap();

    let e = load_err(&dir);
    assert_eq!(e.file, dir.join("idle.png"));
    assert!(!e.reason.is_empty());
}

#[test]
fn width_not_a_multiple_of_height_names_the_png() {
    let dir = fixture("bad-width");
    write_png(&dir.join("idle.png"), 100, 64);
    fs::write(dir.join("character.toml"), "[idle]\nfps = 8\nloop = true\n").unwrap();

    let e = load_err(&dir);
    assert_eq!(e.file, dir.join("idle.png"));
    assert!(e.reason.contains("multiple"), "{}", e.reason);
}

#[test]
fn malformed_manifest_names_character_toml() {
    let dir = fixture("malformed");
    write_png(&dir.join("idle.png"), 64, 64);
    fs::write(dir.join("character.toml"), "[idle\nfps = eight\n").unwrap();

    let e = load_err(&dir);
    assert_eq!(e.file, dir.join("character.toml"));
    assert!(!e.reason.is_empty());
}

#[test]
fn animation_missing_from_manifest_names_character_toml() {
    let dir = fixture("not-in-manifest");
    write_png(&dir.join("idle.png"), 64, 64);
    fs::write(dir.join("character.toml"), "[walk]\nfps = 8\nloop = true\n").unwrap();

    let e = load_err(&dir);
    assert_eq!(e.file, dir.join("character.toml"));
    assert!(e.reason.contains("idle"), "{}", e.reason);
}

#[test]
fn opaque_rgb_png_loads_as_fully_opaque_rgba() {
    let dir = fixture("rgb");
    let file = fs::File::create(dir.join("idle.png")).unwrap();
    let mut enc = png::Encoder::new(file, 2, 1);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .unwrap()
        .write_image_data(&[10, 20, 30, 40, 50, 60])
        .unwrap();
    fs::write(dir.join("character.toml"), "[idle]\nfps = 8\nloop = true\n").unwrap();

    let idle = &assets::load(&dir).unwrap()["idle"];
    assert_eq!(idle.clip.frames, 2);
    assert_eq!(idle.rgba, [10, 20, 30, 255, 40, 50, 60, 255]);
}
