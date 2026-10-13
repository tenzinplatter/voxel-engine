use gl33::{global_loader::*, *};

#[derive(Clone, Copy)]
pub(crate) struct Texture {
    id: u32,
}

impl Texture {
    /// Creates a new texture object.
    pub(crate) fn new() -> Option<Self> {
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

    /// Assumes a y=0 -> top formatted image, will flip it
    pub(crate) fn set_from_pixels(pixels: Vec<u8>, width_px: usize, height_px: usize) {
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
    pub(crate) fn bind(&self) {
        unsafe {
            glBindTexture(GL_TEXTURE_2D, self.id);
        }
    }
}
