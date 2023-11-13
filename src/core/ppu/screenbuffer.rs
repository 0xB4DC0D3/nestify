pub struct ScreenBuffer {
    width: usize,
    buffer: Vec<u8>,
}

impl ScreenBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            buffer: vec![0; width * height],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        let index = y * self.width + x;
        
        if index < self.buffer.len() {
            self.buffer[index] = color;
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        let index = y * self.width + x;
        
        self.buffer[index]
    }
}
