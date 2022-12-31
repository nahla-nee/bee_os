use core::{fmt, cell::OnceCell};

use bootloader_api::info::{FrameBuffer, FrameBufferInfo};
use noto_sans_mono_bitmap::{get_raster, get_raster_width, FontWeight, RasterHeight};
use spin::Mutex;

/// (R, G, B) color
#[derive(Clone, Copy)]
pub struct Color(pub u8, pub u8, pub u8);

/// (x, y) coordinate
#[derive(Clone, Copy)]
pub struct Point(pub usize, pub usize);

pub static TEXT_DISPLAY: Mutex<OnceCell<TextDisplay>> = Mutex::new(OnceCell::new());

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::display::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::display::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! clearscrn {
    () => ($crate::display::_clearscrn());
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    TEXT_DISPLAY.lock().get_mut().expect("Uninitialized TEXT_DISPLAY").write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn _clearscrn() {
    TEXT_DISPLAY.lock().get_mut().expect("Uninitialized TEXT_DISPLAY").clear();
}

pub struct Display {
    fb: &'static mut FrameBuffer,
    fb_info: FrameBufferInfo,
    // this function is unsafe because it doesn't check if the given coordinates
    // are actually within the bounds of the buffer. That's up to the caller to ensure
    draw_pixel_method: unsafe fn(&mut Display, Color, usize),
}

impl Display {
    pub fn new(fb: &'static mut FrameBuffer) -> Display {
        let draw_pixel_rgb = |display: &mut Display, color: Color, position: usize| {
            let fb = display.fb.buffer_mut();
            fb[position] = color.0;
            fb[position + 1] = color.1;
            fb[position + 2] = color.2;
        };
        let draw_pixel_bgr = |display: &mut Display, color: Color, position: usize| {
            let fb = display.fb.buffer_mut();
            fb[position] = color.2;
            fb[position + 1] = color.1;
            fb[position + 2] = color.0;
        };
        let draw_pixel_grayscale = |display: &mut Display, color: Color, position: usize| {
            let fb = display.fb.buffer_mut();
            fb[position] = ((color.0 as u16 + color.1 as u16 + color.2 as u16) / 3) as u8;
        };

        let fb_info = fb.info();

        let draw_pixel_method = match fb_info.pixel_format {
            bootloader_api::info::PixelFormat::Rgb => draw_pixel_rgb,
            bootloader_api::info::PixelFormat::Bgr => draw_pixel_bgr,
            bootloader_api::info::PixelFormat::U8 => draw_pixel_grayscale,
            _ => panic!("unknown pixel format for framebuffer"),
        };

        Display {
            fb,
            fb_info,
            draw_pixel_method,
        }
    }

    fn pixel_pos_to_real(&mut self, position: Point) -> usize {
        let stride = self.fb_info.stride * self.fb_info.bytes_per_pixel;
        stride * position.1 + position.0 * self.fb_info.bytes_per_pixel
    }

    /// This function is unsafe as it does not do bounds checking to determine
    /// if the given pixel coordinates are within the bounds of the framebuffer.
    pub unsafe fn draw_pixel(&mut self, color: Color, position: Point) {
        let position = self.pixel_pos_to_real(position);
        (self.draw_pixel_method)(self, color, position);
    }

    pub fn clear(&mut self, color: Color) {
        self.draw_rect(color, Point(0, 0), Point(self.fb_info.width, self.fb_info.height));
    }

    pub fn draw_rect(&mut self, color: Color, top_left: Point, bottom_right: Point) {
        if top_left.0 > bottom_right.0 {
            panic!("Top left coordinate is further right than the bottom right coordinate");
        }
        if top_left.1 > bottom_right.1 {
            panic!("Top left coordinate is further down than the bottom right coordinate");
        }
        if usize::max(top_left.0, bottom_right.0) > self.fb_info.width {
            panic!("X coordinate is larger than framebuffer");
        }
        if usize::max(top_left.1, bottom_right.1) > self.fb_info.height {
            panic!("Y coordinate is larger than framebuffer");
        }

        for y in top_left.1..bottom_right.1 {
            for x in top_left.0..bottom_right.0 {
                unsafe { self.draw_pixel(color, Point(x, y)) }
            }
        }
    }

    pub fn putc(
        &mut self,
        c: char,
        color: Color,
        height: RasterHeight,
        weight: FontWeight,
        position: Point,
    ) {
        // The largest x point this raster will draw to
        let max_x = position.0 + get_raster_width(weight, height);
        // The largest y point this raster will draw to
        let max_y = position.1 + get_raster_height(height);
        if max_x >= self.fb_info.width || max_y >= self.fb_info.height {
            panic!("Given coordinates are outside framebuffer bounds")
        }

        let raster = match get_raster(c, FontWeight::Regular, RasterHeight::Size16) {
            Some(raster) => raster,
            None => get_raster('?', FontWeight::Regular, RasterHeight::Size16).unwrap(),
        };

        for (row_idx, &row) in raster.raster().iter().enumerate() {
            for (col_idx, &intensity) in row.iter().enumerate() {
                let mut color = color;
                color.0 = (color.0 as f32 * (intensity as f32 / 255.0)) as u8;
                color.1 = (color.1 as f32 * (intensity as f32 / 255.0)) as u8;
                color.2 = (color.2 as f32 * (intensity as f32 / 255.0)) as u8;
                unsafe {
                    self.draw_pixel(color, Point(position.0 + col_idx, position.1 + row_idx))
                };
            }
        }
    }

    /// Copies a rectangular chunk from from src to dst using the given width and height.
    /// This function is unsafe as it does not do bound checks to make sure the rectangles
    /// aren't exceeding the size of the framebuffer.
    pub unsafe fn copy_rect(&mut self, src: Point, dst: Point, width: usize, height: usize) {
        // convert from (x, y) coordinates to real pixel position coordinates
        for y in 0..height {
            // we copy one horizontal line at a time
            let src_line_start = self.pixel_pos_to_real(Point(src.0, src.1+y));
            let src_line_end = self.pixel_pos_to_real(Point(src.0+width, src.1+y));
            let dst_line_start = self.pixel_pos_to_real(Point(dst.0, dst.1+y));

            self.fb.buffer_mut().copy_within(src_line_start..src_line_end, dst_line_start);
        }
    }
}

const fn get_raster_height(height: RasterHeight) -> usize {
    match height {
        RasterHeight::Size16 => 16,
        RasterHeight::Size20 => 20,
        RasterHeight::Size24 => 24,
        RasterHeight::Size32 => 32,
    }
}

pub struct TextDisplay {
    // x, y coordinates of column and row, in chars
    cursor: Point,
    // Width in terms of how many chars fit on a line
    width: usize,
    // Height in terms of how many lines fit in the framebuffer
    height: usize,
    // The background color
    clear_color: Color,
    // The text color
    text_color: Color,
    display: Display,
}

impl TextDisplay {
    pub fn new(fb: &'static mut FrameBuffer, clear_color: Color, text_color: Color) -> TextDisplay {
        let display = Display::new(fb);
        let cursor = Point(0, 0);
        let width =
            display.fb_info.stride / get_raster_width(FontWeight::Regular, RasterHeight::Size20);
        let height = display.fb_info.height / get_raster_height(RasterHeight::Size20);

        TextDisplay {
            cursor,
            width,
            height,
            clear_color,
            text_color,
            display,
        }
    }

    const fn raster_width() -> usize {
        get_raster_width(FontWeight::Regular, RasterHeight::Size20)
    }

    const fn raster_height() -> usize {
        get_raster_height(RasterHeight::Size20)
    }

    pub fn move_cursor(&mut self, position: Point) {
        if position.0 > self.width || position.1 > self.height {
            panic!("Attempted to move cursor outside of framebuffer bounds");
        }
        self.cursor = position;
    }

    pub fn increment_cursor_pos(&mut self) {
        self.cursor.0 = (self.cursor.0 + 1) % self.width;
        if self.cursor.0 == 0 {
            self.cursor.1 += 1;
            if self.cursor.1 == self.height {
                self.scroll_down();
            }
        }
    }

    pub fn cursor_new_line(&mut self) {
        self.cursor.0 = 0;
        self.cursor.1 += 1;
        if self.cursor.1 == self.height {
            self.scroll_down();
        }
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    pub fn set_text_color(&mut self, color: Color) {
        self.text_color = color;
    }

    pub fn clear(&mut self) {
        self.display.clear(self.clear_color);
    }

    pub fn clear_line(&mut self, line: usize) {
        if line >= self.height {
            panic!("Line index is greater than the maximum allowed index!");
        }

        let min_y = line * Self::raster_height();
        let max_y = min_y + Self::raster_height();
        self.display.draw_rect(self.clear_color, Point(0, min_y),
            Point(self.display.fb_info.width, max_y));
    }

    pub fn write_text(&mut self, text: &str) {
        let mut lines = text.lines().peekable();
        while let Some(line) = lines.next() {
            for c in line.chars() {
                let x = self.cursor.0 * Self::raster_width();
                let y = self.cursor.1 * Self::raster_height();
                self.display
                    .putc(c, self.text_color, RasterHeight::Size20, FontWeight::Regular, Point(x, y));
                self.increment_cursor_pos();
            }
            // we only want to print a newline if this isn't the last line.
            if lines.peek().is_some() {
                self.cursor_new_line();
            }
        }
        // and if it is the last line only start a new line if the last character is newline
        if let Some(c) = text.chars().last() && c == '\n' {
            self.cursor_new_line();
        }
    }

    pub fn scroll_down(&mut self) {
        for line in 1..self.height {
            unsafe {
                self.copy_line(line, line-1);
            }
        }
        self.clear_line(self.height-1);
        if self.cursor.1 != 0 {
            self.cursor.1 -= 1;
        }
    }

    pub unsafe fn copy_line(&mut self, src: usize, dst: usize) {
        let src_y = src * Self::raster_height();
        let dst_y = dst * Self::raster_height();
        self.display.copy_rect(Point(0, src_y), Point(0, dst_y), Self::raster_width()*self.width,
            Self::raster_height())
    }
}

impl fmt::Write for TextDisplay {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_text(s);
        Ok(())
    }
}