use anyhow::{Context, Result, bail};
use glam::{UVec2, Vec2};
use image::{ImageBuffer, Rgba};
use log::info;
use serde::Deserialize;

use std::{
    collections::BTreeMap,
    fs::read_dir,
    path::{Path, PathBuf},
};

use crate::render::texture::Texture;

const TEXTURES_DIR: &str = "assets/textures/";
const BYTES_PER_PX: usize = 4;
pub const TEXTURE_SIZE_PX: usize = 32;
const TEXTURE_SIZE_PX_WITH_PADDING: usize = TEXTURE_SIZE_PX + 2;

pub struct TextureAtlas {
    pub size: Vec2,
    pub textures: BTreeMap<String, TextureAtlasEntry>,
    pub texture: Texture,
}

/// NOTE: in UV 0 - 1 coords
#[derive(Debug)]
pub struct TextureAtlasEntry {
    pub top_left: Vec2,
    pub size: Vec2,
}

#[derive(Deserialize)]
struct TextureAtlasKeyEntry {
    img: String,
}

impl TextureAtlasEntry {
    /// Returns the four UV coordinates (Vec2) for the corners of the rectangle
    /// Order: top-left, top-right, bottom-right, bottom-left
    pub fn to_uvs(&self) -> [Vec2; 4] {
        let tl = self.top_left;
        let tr = Vec2::new(self.top_left.x + self.size.x, self.top_left.y);
        let br = Vec2::new(self.top_left.x + self.size.x, self.top_left.y + self.size.y);
        let bl = Vec2::new(self.top_left.x, self.top_left.y + self.size.y);
        [tl, tr, br, bl]
    }
}

impl TextureAtlas {
    /// NOTE: assumes all textures are 16x16
    pub fn try_parse_atlas() -> Result<Self> {
        let textures = get_textures()?;

        let (atlas_pixels_buf, (atlas_width_px, atlas_height_px)) =
            generate_texture_atlas_pixels(&textures)?;
        let textures = get_textures_as_uv(textures, atlas_width_px, atlas_height_px);

        let dbg_img_dir = Path::new("/home/tenzin/.cache/voxel-engine");
        if dbg_img_dir.is_dir() {
            std::fs::create_dir_all(dbg_img_dir).context("Failed to create dbg img dir")?;
        }

        write_image_from_pixels(&atlas_pixels_buf, dbg_img_dir.join("atlas.png"));

        let texture = match Texture::new() {
            Some(t) => t,
            None => bail!("Failed to create texture"),
        };

        texture.bind();
        Texture::set_from_pixels(atlas_pixels_buf, atlas_width_px, atlas_height_px);

        Ok(Self {
            size: Vec2::new(atlas_width_px as f32, atlas_height_px as f32),
            textures,
            texture,
        })
    }
}

fn get_textures_as_uv(
    textures: BTreeMap<String, TextureAtlasKeyEntry>,
    atlas_width_px: usize,
    atlas_height_px: usize,
) -> BTreeMap<String, TextureAtlasEntry> {
    textures
        .into_iter()
        .enumerate()
        .map(|(i, (name, _entry))| {
            let inset_px = 0.5;
            // assumes square atlas
            let inset_percent = inset_px / atlas_width_px as f32;

            let top_left_px = [
                (i as f32 % atlas_width_px as f32) * TEXTURE_SIZE_PX as f32,
                (i as f32 / atlas_width_px as f32).floor() * TEXTURE_SIZE_PX as f32,
            ];

            let top_left = Vec2::from_array([
                top_left_px[0] / atlas_width_px as f32,
                top_left_px[1] / atlas_height_px as f32,
            ]);

            // size in uv
            let size = Vec2::new(
                TEXTURE_SIZE_PX as f32 / atlas_width_px as f32,
                TEXTURE_SIZE_PX as f32 / atlas_height_px as f32,
            );

            // inset each by inset_px
            let top_left = top_left * (1. + inset_percent);
            let size = size * (1. - inset_percent);

            (name, TextureAtlasEntry { top_left, size })
        })
        .collect()
}

/// @return (pixel_buffer, (width, height))
fn generate_texture_atlas_pixels(
    textures: &BTreeMap<String, TextureAtlasKeyEntry>,
) -> Result<(Vec<u8>, (usize, usize))> {
    // width and height in pixels
    let (width_px, height_px) = (1024, 1024);
    let size = width_px * height_px * 4;
    let mut pixel_buf: Vec<u8> = vec![255; size];

    for (i, (name, tex)) in textures.iter().enumerate() {
        let img = image::open(&tex.img)?;

        if img.width() != TEXTURE_SIZE_PX as u32 || img.height() != TEXTURE_SIZE_PX as u32 {
            bail!(
                "img {} is not {}x{} (got size {}x{})",
                name,
                TEXTURE_SIZE_PX,
                TEXTURE_SIZE_PX,
                img.width(),
                img.height()
            );
        }

        let rgba_img = img.to_rgba8();
        let rgba_buf = rgba_img.as_raw();

        // assert buf is 4 bytes per px
        if rgba_buf.len() != TEXTURE_SIZE_PX * TEXTURE_SIZE_PX * 4 {
            bail!(
                "img buf is not {} pixels (got {})",
                TEXTURE_SIZE_PX * TEXTURE_SIZE_PX * 4,
                rgba_buf.len()
            );
        }

        let top_left_px = UVec2::new(
            (i % width_px) as u32 * TEXTURE_SIZE_PX as u32,
            (i / height_px) as u32,
        );

        // TODO: write padding to each texture to remove border artifacts
        for n in 0..TEXTURE_SIZE_PX {
            // standard offset for 2D coord -> 1D
            let dest_offset_px = (top_left_px.x
                + (top_left_px.y * TEXTURE_SIZE_PX as u32 * width_px as u32))
                as usize;
            let dest_offset = (dest_offset_px + (n * width_px)) * BYTES_PER_PX;

            // same thing, we just are always copying blocks of 16 (a row)
            let src_offset = (n * TEXTURE_SIZE_PX) * BYTES_PER_PX;

            let nbytes = TEXTURE_SIZE_PX * BYTES_PER_PX;

            let dest = &mut pixel_buf[dest_offset..dest_offset + nbytes];
            let src = &rgba_buf[src_offset..src_offset + nbytes];

            dest.copy_from_slice(src);
        }
    }

    Ok((pixel_buf, (width_px, height_px)))
}

fn write_image_from_pixels<P: AsRef<Path>>(pixels: &[u8], path: P) {
    // Assumes the image is square and 4 bytes per pixel (RGBA)
    let side = (pixels.len() / 4).isqrt() as u32;
    let img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(side, side, pixels.to_vec())
        .expect("Failed to create image buffer from raw pixels");
    img.save(path).expect("Failed to save image");
}

fn get_texture_dirs() -> Result<Vec<PathBuf>> {
    Ok(read_dir(TEXTURES_DIR)
        .context(format!("texture dir: {TEXTURES_DIR}"))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|dir| dir.join("key.json").is_file())
        .collect())
}

fn get_textures() -> Result<BTreeMap<String, TextureAtlasKeyEntry>> {
    let texture_dirs = get_texture_dirs()?;
    let mut textures: BTreeMap<String, TextureAtlasKeyEntry> = BTreeMap::new();
    for dir in &texture_dirs {
        info!("Found texture dir: {:?}", dir);
        let key_path = dir.join("key.json");
        let contents = std::fs::read_to_string(&key_path)
            .with_context(|| format!("Failed to read texture atlas key at {:?}", key_path))?;
        let entries = serde_json::from_str::<BTreeMap<String, TextureAtlasKeyEntry>>(&contents)
            .with_context(|| format!("Failed to parse texture atlas key at {:?}", key_path))?;

        for entry in entries {
            let texture_path = dir.join("imgs").join(&entry.1.img);
            if !texture_path.is_file() {
                bail!("Texture image file {:?} does not exist", texture_path);
            }
            textures.insert(
                entry.0,
                TextureAtlasKeyEntry {
                    img: texture_path.to_string_lossy().to_string(),
                },
            );
        }
    }

    Ok(textures)
}
