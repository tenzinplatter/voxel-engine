use gl33::{global_loader::*, *};

#[derive(Clone, Copy)]
pub struct Texture {
    pub id: u32,
}

impl Texture {
    /// Creates a new texture object.
    pub fn new() -> Option<Self> {
        let mut texture_id: u32 = 0;
        unsafe {
            glGenTextures(1, &mut texture_id);
        }

        if texture_id != 0 {
            Some(Texture { id: texture_id })
        } else {
            None
        }
    }

    /// Loads an image from the given path and sets it as the texture data.
    pub fn set_image(img_path: &str) {
        let img = image::open(img_path).expect("Failed to load texture image");
        // Convert to RGBA8 and flip vertically (OpenGL expects bottom-left origin)
        let rgba_img = img.to_rgba8();
        // TODO: maybe a way to do this without cloning in future? idk ill probs be removing this
        // fn anyway
        let pixels = rgba_img.to_vec();

        Texture::set_from_pixels(pixels, rgba_img.width() as usize, rgba_img.height() as usize);
    }

    /// Assumes a y=0 -> top formatted image, will flip it
    pub fn set_from_pixels(pixels: Vec<u8>, width_px: usize, height_px: usize) {
        unsafe {
            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                GL_RGBA.0 as i32,
                width_px as i32,
                height_px as i32,
                0,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                pixels.as_ptr().cast(),
            );
            glGenerateMipmap(GL_TEXTURE_2D);
        }
    }

    /// Binds this texture as the active texture.
    pub fn bind(&self) {
        unsafe {
            glBindTexture(GL_TEXTURE_2D, self.id);
        }
    }
}

/// Configures default texture parameters (wrapping and filtering).
pub fn setup_texture_opts() {
    unsafe {
        // set texture wrapping to mirrored repeat
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_MIRRORED_REPEAT.0 as i32);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_MIRRORED_REPEAT.0 as i32);

        // set texture filtering to nearest for minification and linear for magnification
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST.0 as i32);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR.0 as i32);

        // set texture filtering to use mipmaps
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR_MIPMAP_LINEAR.0 as i32);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR.0 as i32);
    }
}
