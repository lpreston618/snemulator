use anyhow::Result;
use egui::IntoAtoms;
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

pub fn draw_checkerboard(pixel_buffer: &mut [u8], buffer_width: usize, tile_width: usize, tile_height: usize, color1: [u8; 4], color2: [u8; 4]) -> Result<()> {
    if (pixel_buffer.len() / buffer_width) * buffer_width != pixel_buffer.len() {
        return Err(anyhow::anyhow!("buffer_width must divide pixel buffer length"));
    }

    let colors = [color1, color2];
    
    pixel_buffer.chunks_mut(buffer_width)
        .enumerate()
        .for_each(|(y, pixel_layer)| {
            pixel_layer.chunks_mut(tile_width)
                .enumerate()
                .for_each(|(x, tile)| {
                    let color_select = ((y / tile_height) + (x / tile_width)) & 1;
                    let pixel_color = colors[color_select];

                    tile.chunks_mut(4)
                        .for_each(|pixel| {
                            pixel[0] = pixel_color[0];
                            pixel[1] = pixel_color[1];
                            pixel[2] = pixel_color[2];
                            pixel[3] = pixel_color[3];
                        });
                });
        });

    Ok(())
}

pub fn draw_diagonal_stripes(pixel_buffer: &mut [u8], buffer_width: usize, stripe_width: usize, stripe_spacing: usize, stripe_color: [u8; 4], spacing_color: [u8; 4]) -> Result<()> {
    if (pixel_buffer.len() / buffer_width) * buffer_width != pixel_buffer.len() {
        return Err(anyhow::anyhow!("buffer_width must divide pixel buffer length"));
    }
    
    let buffer_height = pixel_buffer.len() / buffer_width;
    
    let period_width = stripe_width + stripe_spacing;
    let full_periods_per_layer = (buffer_width - 1) / period_width;
    
    let mut layer: Vec<u8> = spacing_color.into_iter()
        .cycle()
        .take(4 * stripe_spacing)
        .collect::<Vec<u8>>();
    
    layer.extend(
        stripe_color.into_iter()
            .cycle()
            .take(4 * stripe_width)
        );
    
    let layer = layer.repeat(full_periods_per_layer + 1);

    for y in 0..buffer_height {
        let offset = y % period_width;

        let start_idx = y * buffer_width * 4;
        let end_idx = (y+1) * buffer_width * 4;

        let layer_start_idx = 4 * offset;
        let layer_end_idx = 4 * (offset + buffer_width);

        pixel_buffer[start_idx..end_idx].copy_from_slice(&layer[layer_start_idx..layer_end_idx]);
    }

    Ok(())
}