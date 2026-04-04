use anyhow::Result;
use glow::HasContext;

pub struct Texture {
    texture: glow::Texture,
    texture_id: egui::TextureId,
    gl: std::sync::Arc<glow::Context>,
    width: usize,
    height: usize,
    max_size: usize,
}

impl Texture {
    pub fn new(painter: &mut egui_glow::Painter, max_width: usize, max_height: usize) -> Self {
        let gl = painter.gl().clone();
         
        let texture = unsafe {
            let tex = gl.create_texture().expect("Failed to create CHR texture");
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
            tex
        };
        
        let texture_id = painter.register_native_texture(texture);
        
        Self {
            texture,
            texture_id,
            gl,
            width: max_width,
            height: max_height,
            max_size: max_width * max_height,
        }
    }
    
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    
    pub fn resize(&mut self, width: usize, height: usize) -> Result<()> {
        if width * height * 4 > self.max_size {
            return Err(anyhow::anyhow!("Texture size must not exceed max_width * max_height"));
        }
        
        self.width = width;
        self.height = height;
        
        Ok(())
    }
    
    pub fn texture_id(&self) -> egui::TextureId {
        self.texture_id
    }
    
    pub fn update_texture(&mut self, pixels: &[u8]) {
        let gl = &self.gl;
        
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D, 0,
                glow::RGBA as i32,
                self.width as i32, self.height as i32,
                0, glow::RGBA, glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(pixels)),
            );
        }
    }
}