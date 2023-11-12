use std::time::{Instant, Duration};

use sdl2::{
    *,
    render::{TextureCreator, Texture},
    video::WindowContext,
    pixels::PixelFormatEnum,
};

use crate::core::ppu::Ppu;

use super::{videobuffer::VideoBuffer, palette::PALETTE};

pub struct Window {
    context: Sdl,
    canvas: render::Canvas<video::Window>,
    videobuffer: VideoBuffer,
}

impl Window {
    pub fn new() -> Self {
        let context = sdl2::init()
            .expect("Unable to create context!");

        let video_subsystem = context
            .video()
            .expect("Unable to create video subsystem!");

        let window = video_subsystem
            .window("Nestify", 256 * 3, 240 * 3)
            .position_centered()
            .build()
            .expect("Unable to create window!");

        let mut canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .expect("Unable to create canvas!");

        canvas
            .set_scale(3.0, 3.0)
            .expect("Unable to set scale for canvas!");

        Self {
            context,
            canvas,
            videobuffer: VideoBuffer::new(256, 240),
        }
    }

    pub fn render(&mut self, ppu: &Ppu) {
        let target_fps: u32 = 120;
        let frame_duration = Duration::from_secs(1) / target_fps;
        let last_frame_time = Instant::now();

        self.clear();

        let screen_buffer = ppu.get_screen_buffer();

        for x in 0..256 {
            for y in 0..240 {
                let (_, color) = screen_buffer.get_pixel(
                    x,
                    y
                );

                self.videobuffer.set_pixel(
                    x,
                    y,
                    PALETTE[color as usize]
                );
            }
        }

        let texture_creator = self.texture_creator();
        let mut texture = texture_creator
            .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
            .expect("Unable to create texture!");

        texture
            .update(None, self.videobuffer.get(), 256 * 3)
            .expect("Unable to update texture!");

        self.update_canvas(&texture);
        self.present();
        
        let elapsed_time = last_frame_time.elapsed();
        if elapsed_time < frame_duration {
            let sleep_time = frame_duration - elapsed_time;
            //std::thread::sleep(sleep_time);
        }
    }

    pub fn clear(&mut self) {
        self.canvas.clear();
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }

    pub fn texture_creator(&self) -> TextureCreator<WindowContext> {
        self.canvas.texture_creator()
    }

    pub fn update_canvas(&mut self, texture: &Texture) {
        self.canvas
            .copy(texture, None, None)
            .expect("Unable to copy texture into canvas!");
    }

    pub fn event_pump(&mut self) -> EventPump {
        self.context
            .event_pump()
            .expect("Unable to get event pump!")
    }
}
