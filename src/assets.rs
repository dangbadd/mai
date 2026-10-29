use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::sim::Clip;

/// Animations every asset folder must provide.
pub const REQUIRED: &[&str] = &["idle", "walk", "sit", "react", "dragged", "fall", "land"];

pub struct Animation {
    pub clip: Clip,
    /// Width and height of one square frame, in pixels.
    pub frame_size: u32,
    /// The whole strip as straight (non-premultiplied) RGBA8, row-major.
    pub rgba: Vec<u8>,
}

#[derive(Debug)]
pub struct AssetError {
    pub file: PathBuf,
    pub reason: String,
}

#[derive(serde::Deserialize)]
struct Entry {
    fps: u32,
    #[serde(rename = "loop")]
    looping: bool,
}

/// Reads `character.toml` and one `<name>.png` strip per required Animation.
pub fn load(dir: &Path) -> Result<HashMap<String, Animation>, AssetError> {
    let manifest_path = dir.join("character.toml");
    let text = std::fs::read_to_string(&manifest_path).map_err(|e| err(&manifest_path, e))?;
    let manifest: HashMap<String, Entry> =
        toml::from_str(&text).map_err(|e| err(&manifest_path, e))?;

    REQUIRED
        .iter()
        .map(|&name| {
            let entry = manifest
                .get(name)
                .ok_or_else(|| err(&manifest_path, format!("missing [{name}] section")))?;
            let png_path = dir.join(format!("{name}.png"));
            let (width, height, rgba) = read_rgba(&png_path).map_err(|e| err(&png_path, e))?;
            if width % height != 0 {
                let reason = format!("width {width} is not a multiple of height {height}");
                return Err(err(&png_path, reason));
            }
            if entry.fps == 0 {
                return Err(err(
                    &manifest_path,
                    format!("[{name}] fps must be greater than 0"),
                ));
            }
            let clip = Clip {
                frames: width / height,
                fps: entry.fps,
                looping: entry.looping,
            };
            Ok((
                name.to_string(),
                Animation {
                    clip,
                    frame_size: height,
                    rgba,
                },
            ))
        })
        .collect()
}

fn read_rgba(path: &Path) -> Result<(u32, u32, Vec<u8>), Box<dyn std::error::Error>> {
    let mut decoder = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path)?));
    // Expand palette/low bit depths and 16-bit channels to 8-bit, then add alpha below if missing.
    decoder.set_transformations(
        png::Transformations::normalize_to_color8() | png::Transformations::ALPHA,
    );
    let mut reader = decoder.read_info()?;
    let mut buf = vec![0; reader.output_buffer_size().ok_or("image too large")?];
    let info = reader.next_frame(&mut buf)?;
    buf.truncate(info.buffer_size());
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::GrayscaleAlpha => buf
            .chunks(2)
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        other => return Err(format!("unsupported color type {other:?}").into()),
    };
    Ok((info.width, info.height, rgba))
}

fn err(file: &Path, reason: impl std::fmt::Display) -> AssetError {
    AssetError {
        file: file.to_path_buf(),
        reason: reason.to_string(),
    }
}
