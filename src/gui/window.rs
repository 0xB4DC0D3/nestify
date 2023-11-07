use std::time::{Instant, Duration};

use sdl2::{
    *,
    render::{TextureCreator, Texture},
    video::WindowContext,
    pixels::PixelFormatEnum,
    rect::Rect
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
            //.present_vsync()
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

    fn get_palette_table(&self, ppu: &Ppu, nametable_index: u16, row: usize, column: usize) -> Vec<u8> {
        let attribute_table = ppu.get_attribute_table(nametable_index);
        let attribute = attribute_table
            .get(row / 4 * 8 + column / 4)
            .cloned()
            .expect("Invalid Attribute table!");

        let palette_index = match (column % 4 / 2, row % 4 / 2) {
            (0, 0) => (attribute >> 0) & 0b11,
            (1, 0) => (attribute >> 2) & 0b11,
            (0, 1) => (attribute >> 4) & 0b11,
            (1, 1) => (attribute >> 6) & 0b11,
            _ => panic!("Invalid palette index!"),
        } as u16;

        ppu.get_palette_table(palette_index)
    }

    fn render_buffer(
        &mut self,
        ppu: &Ppu,
        buffer: &Vec<u8>,
        viewport: Rect,
        shift_x: u16,
        shift_y: u16,
        nametable_index: u16,
        second_nametable_index: u16,
        main_viewport: bool
    ) {
        buffer
            .chunks(64)
            .enumerate()
            .for_each(|(index, tile)| {
                let column = index % 32 * 8;
                let row = index / 32 * 8;
                let current_nametable_index = match main_viewport {
                    true => nametable_index,
                    false => second_nametable_index,
                };
                let palette_table = self.get_palette_table(ppu, current_nametable_index, index / 32, index % 32);

                tile.iter()
                    .enumerate()
                    .for_each(|(index, &pixel)| {
                        let x = index % 8;
                        let y = index / 8;

                        let pixel_x = column + x;
                        let pixel_y = row + y;

                        let color = match pixel {
                            0 => PALETTE[palette_table[3] as usize],
                            1 => PALETTE[palette_table[0] as usize],
                            2 => PALETTE[palette_table[1] as usize],
                            3 => PALETTE[palette_table[2] as usize],
                            _ => panic!("Invalid pixel data!"),
                        };

                        let in_viewport_x = pixel_x as i32 >= viewport.x() && pixel_x < viewport.width() as usize;
                        let in_viewport_y = pixel_y as i32 >= viewport.y() && pixel_y < viewport.height() as usize;

                        let (pos_x, pos_y) = match main_viewport {
                            true => (pixel_x - shift_x as usize, pixel_y - shift_y as usize),
                            false => (pixel_x + shift_x as usize, pixel_y + shift_y as usize),
                        };

                        if in_viewport_x && in_viewport_y {
                            self.videobuffer.set_pixel(
                                pos_x,
                                pos_y,
                                color
                            );
                        }
                    });
            });
    }

    pub fn render(&mut self, ppu: &Ppu) {
        let target_fps: u32 = 60;
        let frame_duration = Duration::from_secs(1) / target_fps;
        let last_frame_time = Instant::now();

        self.clear();

        let (main_buffer, second_buffer) = ppu.get_background_buffer();
        let (scroll_x, scroll_y) = ppu.get_scroll();

        let nametable_index = ppu.get_nametable_index();
        let second_nametable_index = ppu.get_second_nametable_index(nametable_index);

        self.render_buffer(
            ppu, 
            &main_buffer,
            Rect::new(scroll_x as i32, scroll_y as i32, 256, 240),
            scroll_x as u16,
            scroll_y as u16,
            nametable_index,
            second_nametable_index,
            true
        );

        match (scroll_x > 0, scroll_y > 0) {
            (true, false) => {
                self.render_buffer(
                    ppu, 
                    &second_buffer,
                    Rect::new(0, 0, scroll_x as u32, 240),
                    256 - scroll_x as u16,
                    0,
                    nametable_index,
                    second_nametable_index,
                    false
                );
            },
            (false, true) => {
                self.render_buffer(
                    ppu,
                    &second_buffer,
                    Rect::new(0, 0, 256, scroll_y as u32),
                    0,
                    240 - scroll_y as u16,
                    nametable_index,
                    second_nametable_index,
                    false,
                );
            },
            (true, true) => {
                self.render_buffer(
                    ppu,
                    &second_buffer,
                    Rect::new(0, 0, scroll_x as u32, scroll_y as u32),
                    256 - scroll_x as u16,
                    240 - scroll_y as u16,
                    nametable_index,
                    second_nametable_index,
                    false
                );
            },
            _ => (),
        };

        let spr_pattern_table_index = ppu.get_spr_patter_table_index();
        let spr_pattern_table = ppu.get_pattern_table(spr_pattern_table_index);

        ppu.get_sprites_buffer()
            .chunks(4)
            .rev()
            .for_each(|metadata| {
                let x_coord = metadata[3] as usize;
                let y_coord = metadata[0] as usize;

                let flip_horizontally = (metadata[2] >> 6) & 0x1 == 0x1;
                let flip_vertically = (metadata[2] >> 7) & 0x1 == 0x1;
                let show_sprite = (metadata[2] >> 5) & 0x1 == 0;

                let tile_index = metadata[1] as usize * 64;

                let palette_index = metadata[2] as u16 & 0b11;
                let palette_table = ppu.get_sprite_palette_table(palette_index);

                // if show_sprite {
                    spr_pattern_table
                        .get(tile_index..tile_index + 64)
                        .into_iter()
                        .flatten()
                        .enumerate()
                        .for_each(|(index, &pixel)| {
                            let x = index % 8;
                            let y = index / 8;

                            let x = if flip_horizontally { 7 - x } else { x };
                            let y = if flip_vertically { 7 - y } else { y };

                            match pixel {
                                0 => (),
                                1 | 2 | 3 => {
                                    self.videobuffer
                                        .set_pixel(
                                            x_coord + x,
                                            y_coord + y,
                                            PALETTE[palette_table[pixel as usize - 1] as usize]
                                    );
                                },
                                _ => panic!("Invalid pixel data!"),
                            }
                        });
                //}
            });

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
            std::thread::sleep(sleep_time);
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
