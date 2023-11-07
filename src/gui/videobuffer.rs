use sdl2::pixels::Color;

pub struct VideoBuffer {
    width: usize,
    buffer: Vec<u8>,
}

impl VideoBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            buffer: vec![0; width * height * 3], // R, G, B
        }
    }

    pub fn get(&self) -> &Vec<u8> {
        &self.buffer
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        let index = y * self.width * 3 + x * 3;
        
        if index + 2 < self.buffer.len() {
            self.buffer[index..index + 3]
                .copy_from_slice(&[color.r, color.g, color.b]);
        }
    }
}
