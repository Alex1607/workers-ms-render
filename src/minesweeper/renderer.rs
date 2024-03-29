use std::collections::BTreeMap;
use std::io::Cursor;
use std::time::Duration;

use gif::{Encoder, Frame as GifFrame, Repeat};
use image::{Delay, DynamicImage, Frame, GenericImage, ImageBuffer, Rgba};

use crate::minesweeper::error::MinesweeperError;
use crate::minesweeper::minesweeper_logic::{Board, FieldState};
use crate::minesweeper::parsers::parser::{ActionType, FlagAction, Metadata, OpenAction};
use crate::minesweeper::textures::load_textures;

pub struct Renderer {
    pub(crate) metadata: Metadata,
    game_board: Board,
    open_data: Vec<OpenAction>,
    flag_data: Vec<FlagAction>,
    image_data: Imagedata,
}

#[derive(Copy, Clone)]
pub enum RenderType {
    Image,
    Gif,
}

impl std::str::FromStr for RenderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_ref() {
            "image" => Ok(RenderType::Image),
            "gif" => Ok(RenderType::Gif),
            _ => Err(format!("Unknown render type: {}", s)),
        }
    }
}

struct Imagedata {
    zero: ImageBuffer<Rgba<u8>, Vec<u8>>,
    one: ImageBuffer<Rgba<u8>, Vec<u8>>,
    two: ImageBuffer<Rgba<u8>, Vec<u8>>,
    three: ImageBuffer<Rgba<u8>, Vec<u8>>,
    four: ImageBuffer<Rgba<u8>, Vec<u8>>,
    five: ImageBuffer<Rgba<u8>, Vec<u8>>,
    six: ImageBuffer<Rgba<u8>, Vec<u8>>,
    seven: ImageBuffer<Rgba<u8>, Vec<u8>>,
    eight: ImageBuffer<Rgba<u8>, Vec<u8>>,
    tnt: ImageBuffer<Rgba<u8>, Vec<u8>>,
    empty: ImageBuffer<Rgba<u8>, Vec<u8>>,
    flag: ImageBuffer<Rgba<u8>, Vec<u8>>,
    unsure_flag: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

impl Imagedata {
    pub fn new(sprite_data: &[u8]) -> Imagedata {
        let im = &mut image::load_from_memory(sprite_data).expect("Custom Textures file not found");

        let zero = im.sub_image(0, 0, 32, 32).to_image();
        let one = im.sub_image(32, 0, 32, 32).to_image();
        let two = im.sub_image(32 * 2, 0, 32, 32).to_image();
        let three = im.sub_image(32 * 3, 0, 32, 32).to_image();
        let four = im.sub_image(32 * 4, 0, 32, 32).to_image();
        let five = im.sub_image(32 * 5, 0, 32, 32).to_image();
        let six = im.sub_image(32 * 6, 0, 32, 32).to_image();
        let seven = im.sub_image(32 * 7, 0, 32, 32).to_image();
        let eight = im.sub_image(32 * 8, 0, 32, 32).to_image();
        let tnt = im.sub_image(32 * 9, 0, 32, 32).to_image();
        let empty = im.sub_image(32 * 10, 0, 32, 32).to_image();
        let flag = im.sub_image(32 * 11, 0, 32, 32).to_image();
        let unsure_flag = im.sub_image(32 * 12, 0, 32, 32).to_image();

        Imagedata {
            zero,
            one,
            two,
            three,
            four,
            five,
            six,
            seven,
            eight,
            tnt,
            empty,
            flag,
            unsure_flag,
        }
    }
}

impl Renderer {
    pub fn new(
        metadata: Metadata,
        game_board: Board,
        open_data: Vec<OpenAction>,
        flag_data: Vec<FlagAction>,
        gif: &bool,
    ) -> Renderer {
        Renderer {
            metadata,
            game_board,
            open_data,
            flag_data,
            image_data: Imagedata::new(load_textures(gif).as_slice()),
        }
    }

    pub fn render_jpeg(&mut self) -> Result<Vec<u8>, MinesweeperError> {
        self.flag_data
            .iter()
            .for_each(|action| action.perform_action(&mut self.game_board));

        self.open_data.iter().for_each(|action| {
            self.game_board
                .open_field(action.x as usize, action.y as usize);
        });

        let percentage_done = self.game_board.calculate_done_percentage();
        let frame = self.generate_image(percentage_done)?;

        let mut buffer = Cursor::new(vec![]);

        DynamicImage::ImageRgba8(frame)
            .write_to(&mut buffer, image::ImageFormat::Png)
            .unwrap();

        Ok(buffer.into_inner())
    }

    pub fn render_gif(&mut self) -> Result<Vec<u8>, MinesweeperError> {
        let mut frames = Vec::new();

        let tick_map: BTreeMap<i64, Vec<ActionType>> = self.create_tick_map();

        let frame = self.generate_image(0)?;
        frames.push(Frame::from_parts(
            frame,
            0,
            0,
            Delay::from_saturating_duration(Duration::from_secs(1)),
        ));

        for (id, tick) in tick_map.iter().enumerate() {
            let next_tick = tick_map.keys().nth(id + 1);

            let duration = if let Some(next) = next_tick {
                Duration::from_millis(((next - tick.0) * self.metadata.timeunits as i64) as u64)
            } else {
                Duration::from_secs(15)
            };

            if tick.1.contains(&ActionType::Flag) {
                self.flag_data
                    .iter()
                    .filter(|flag| flag.total_time.eq(tick.0))
                    .for_each(|flag| flag.perform_action(&mut self.game_board));
                //Remove all elements which are less than tick.0
                self.flag_data.retain(|flag| flag.total_time.gt(tick.0))
            }

            if tick.1.contains(&ActionType::Open) {
                self.open_data
                    .iter()
                    .filter(|flag| flag.total_time.eq(tick.0))
                    .for_each(|action| {
                        self.game_board
                            .open_field(action.x as usize, action.y as usize);
                    });

                //Remove all elements which are less than tick.0
                self.open_data.retain(|open| open.total_time.gt(tick.0))
            }

            let frame = self.generate_image(if id == (tick_map.len() - 1) {
                100
            } else {
                ((id as f32 / tick_map.len() as f32) * 100.0) as u32
            })?;

            frames.push(Frame::from_parts(
                frame,
                0,
                0,
                Delay::from_saturating_duration(duration),
            ));
        }

        self.encode_frames_to_gif(frames)
    }

    fn encode_frames_to_gif(&mut self, frames: Vec<Frame>) -> Result<Vec<u8>, MinesweeperError> {
        if frames.is_empty() {
            return Err(MinesweeperError::NoFrames);
        }

        let (width, height) = frames.first().unwrap().buffer().dimensions();

        let buffer = &mut Cursor::new(vec![]);

        let mut encoder = Encoder::new(buffer, width as u16, height as u16, &[]).unwrap();

        encoder
            .set_repeat(Repeat::Infinite)
            .map_err(|_| MinesweeperError::GifEncoding)?;

        for (_i, image) in frames.into_iter().enumerate() {
            let frame_delay = image.delay().numer_denom_ms().0 / 10;
            let rbga_frame = &mut image.into_buffer();
            let mut frame = GifFrame::from_rgba_speed(width as u16, height as u16, rbga_frame, 1);
            frame.delay = frame_delay as u16;
            frame.dispose = gif::DisposalMethod::Keep;

            encoder
                .write_frame(&frame)
                .map_err(|_| MinesweeperError::GifEncoding)?;
        }

        Ok(encoder
            .into_inner()
            .map_err(|_| MinesweeperError::GifEncoding)?
            .clone()
            .into_inner())
    }

    fn create_tick_map(&mut self) -> BTreeMap<i64, Vec<ActionType>> {
        let mut tick_map = BTreeMap::new();

        for x in self.open_data.iter() {
            Self::insert_action(&mut tick_map, x.total_time, ActionType::Open)
        }

        for x in self.flag_data.iter() {
            Self::insert_action(&mut tick_map, x.total_time, ActionType::Flag)
        }

        tick_map
    }

    fn insert_action(
        tick_map: &mut BTreeMap<i64, Vec<ActionType>>,
        total_time: i64,
        action: ActionType,
    ) {
        if let std::collections::btree_map::Entry::Vacant(e) = tick_map.entry(total_time) {
            e.insert(vec![action]);
        } else {
            tick_map.get_mut(&total_time).unwrap().push(action);
        }
    }

    fn generate_image(
        &mut self,
        percentage: u32,
    ) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, MinesweeperError> {
        let progressbar_height = 4;
        let imgx = (self.metadata.x_size * 32) as u32;
        let imgy = ((self.metadata.y_size * 32) as u32) + progressbar_height;

        let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

        for x in 0..self.metadata.x_size as u32 {
            for y in 0..self.metadata.y_size as u32 {
                let field = &self.game_board.fields[y as usize][x as usize];

                // Only render fields that got changed in the last iteration
                if !self.game_board.changed_fields[y as usize][x as usize] {
                    continue;
                }

                let xx = x * 32;
                let yy = y * 32;
                if field.field_state == FieldState::Closed {
                    imgbuf
                        .copy_from(&self.image_data.empty, xx, yy)
                        .map_err(|_| MinesweeperError::ImageInsertion)?;
                    continue;
                }
                if field.field_state == FieldState::Flagged {
                    imgbuf
                        .copy_from(&self.image_data.flag, xx, yy)
                        .map_err(|_| MinesweeperError::ImageInsertion)?;
                    continue;
                }
                if field.field_state == FieldState::UnsureFlagged {
                    imgbuf
                        .copy_from(&self.image_data.unsure_flag, xx, yy)
                        .map_err(|_| MinesweeperError::ImageInsertion)?;
                    continue;
                }
                if field.mine {
                    imgbuf
                        .copy_from(&self.image_data.tnt, xx, yy)
                        .map_err(|_| MinesweeperError::ImageInsertion)?;
                    continue;
                }
                match field.value {
                    0 => imgbuf
                        .copy_from(&self.image_data.zero, xx, yy)
                        .map_err(|_| MinesweeperError::ImageInsertion)?,
                    1 => imgbuf
                        .copy_from(&self.image_data.one, xx, yy)
                        .map_err(|_| MinesweeperError::ImageInsertion)?,
                    2 => imgbuf
                        .copy_from(&self.image_data.two, xx, yy)
                        .map_err(|_| MinesweeperError::ImageInsertion)?,
                    3 => imgbuf
                        .copy_from(&self.image_data.three, xx, yy)
                        .map_err(|_| MinesweeperError::ImageInsertion)?,
                    4 => imgbuf
                        .copy_from(&self.image_data.four, xx, yy)
                        .map_err(|_| MinesweeperError::ImageInsertion)?,
                    5 => imgbuf
                        .copy_from(&self.image_data.five, xx, yy)
                        .map_err(|_| MinesweeperError::ImageInsertion)?,
                    6 => imgbuf
                        .copy_from(&self.image_data.six, xx, yy)
                        .map_err(|_| MinesweeperError::ImageInsertion)?,
                    7 => imgbuf
                        .copy_from(&self.image_data.seven, xx, yy)
                        .map_err(|_| MinesweeperError::ImageInsertion)?,
                    8 => imgbuf
                        .copy_from(&self.image_data.eight, xx, yy)
                        .map_err(|_| MinesweeperError::ImageInsertion)?,
                    _ => unreachable!(),
                }
            }
            let pixel_coloring = (percentage * imgx) / 100;

            for x in 0..imgx {
                for y in (imgy - progressbar_height)..imgy {
                    let pixel = imgbuf.get_pixel_mut(x, y);
                    if x <= pixel_coloring {
                        *pixel = Rgba([103, 149, 60, 255]);
                    } else {
                        *pixel = Rgba([0, 0, 0, 255]);
                    }
                }
            }
        }

        //Reset the changed fields after they got rendered
        self.game_board
            .changed_fields
            .iter_mut()
            .for_each(|row| row.iter_mut().for_each(|field| *field = false));

        Ok(imgbuf)
    }
}
