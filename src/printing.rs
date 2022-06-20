use std::{fs::File, path::Path, io::Write};
use image::{Luma, imageops, Pixel};
use crate::bitimage::BitImage;

/// # About
/// Base struct used for printing
/// # Creating
/// On Windows:
/// ```
/// let mut printer = Printer::new("\\\\MACHINE NAME\\SHARED_PRINTER_NAME");
/// ```
pub struct Printer {
  path: String,
  file_handle: File,
}


pub const GS: u8 = 0x1d;
pub const ESC: u8 = 0x1b;

impl Printer {

  /// # Examples
  /// On Windows:
  /// ```
  /// let mut printer = Printer::new("\\\\MACHINE NAME\\SHARED_PRINTER_NAME");
  /// ```
  pub fn new(printer_path: &str) -> Self {
    Printer {
      file_handle: {
        let path = Path::new(printer_path);
        match File::create(path) {
          Ok(handle) => handle,
          Err(e) => panic!("FAILED TO CREATE FILE HANDLE FOR PRINTER {:?}", e)
        }
      },
      path: printer_path.to_string()
    }
  }

  fn write_byte(&mut self, byte: u8) {
    match self.file_handle.write(&[byte]) {
      Ok(_) => (),
      Err(e) => panic!("error: {}", e)
    };
  }

  fn flush_buf(&mut self) {
    match self.file_handle.flush() {
      Ok(_) => (),
      Err(e) => panic!("error: {}", e)
    };
  }

  fn print_buffer(&mut self) {
    match self.file_handle.write(&[0x0c]) {
      Ok(_) => (),
      Err(e) => panic!("error: {}", e)
    };
  }

  /// # About
  /// Turns a u16 into a format that can be sent to the printer
  fn to_two_byte(&self, num: u16) -> [u8;2] {
    let mut bytes: [u8;2] = num.to_be_bytes();
    bytes.reverse();
    bytes
  }

  /// # Examples
  /// ```
  /// printer.println("Hello World!");
  /// ```
  pub fn println(&mut self, message: &str) {
    match self.file_handle.write_all(message.as_bytes()) {
      Ok(_) => (),
      Err(e) => panic!("error: {}", e)
    };
    self.write_byte(0x0c);
    self.flush_buf();
  }

  /// # About
  /// Funcion used to send an array of bytes to the printer and flush its buffer.
  /// # Warning
  /// Sending a sequence of bytes might cause the printer to stop working,
  /// requiring a reboot.
  ///
  /// Only use this if you know what you're doing.
  ///
  /// # Tip
  /// use the constants ``printing::GS`` and ``printing::ESC`` as escape characters.
  pub fn print_bytes(&mut self, message: &[u8]) {
    match self.file_handle.write_all(message) {
      Ok(_) => (),
      Err(e) => panic!("error: {}", e)
    };
    self.flush_buf();
  }

  /// # About
  /// Simply puts the contents of the supplied vector into the buffer.
  ///
  /// Requires flushing.
  fn write_vec(&mut self, bytes: &Vec<u8>) {
    for byte in bytes {
      self.write_byte(*byte);
    }
  }

  /// # About
  /// Must be either "left", "center", or "right" (case insensitive).
  ///
  /// Falls back to "left" if not one of those
  /// # Example
  /// ```
  /// printer.set_justification("center");
  /// ```
  pub fn set_justification(&mut self, value: u8) {
    self.print_bytes(&[ESC, 0x61, value]);
  }

  pub fn set_text_mode(&mut self, double_width: bool, double_height: bool, bold: bool, underline: bool) {
    let mut msg: Vec<u8> = Vec::from([ESC, b'!']);
    let mut settings: u8 = 0;
    if double_width {
      settings |= 0b00100000;
    }
    if double_height {
      settings |= 0b00010000;
    }
    if bold {
      settings |= 0b00001000;
    }
    if underline {
      settings |= 0b00000001;
    }
    msg.push(settings);
    self.write_vec(&msg);
    self.flush_buf();
  }

  pub fn print_qr_code(&mut self, size: u8, data: &[u8]) {

    self.print_bytes(&[GS, 0x28, 0x6b, 0x03, 0x00, 0x31, 0x43, size]);


    let mut cmd: Vec<u8> = Vec::from([GS, 0x28, 0x6b]);
    cmd.extend_from_slice(&self.to_two_byte(data.len() as u16 + 3));
    cmd.extend_from_slice(&[0x31, 0x50, 0x30]);
    cmd.extend_from_slice(data);

    self.write_vec(&cmd);
    self.flush_buf();
  }

  /// # About
  /// Prints bitmaps using the "GS v 0" ESC-POS command
  ///
  /// Bitmaps are read left-to-right, top-to-bottom, with every bit being a dot in the printer
  /// # Examples
  /// ```
  /// // A 16x8 frame
  /// let bitmap: [u8; 8] = [
  ///   0b11111111, 0b11111111,
  ///   0b10000000, 0b00000001,
  ///   0b10000000, 0b00000001,
  ///   0b10000000, 0b00000001,
  ///   0b10000000, 0b00000001,
  ///   0b10000000, 0b00000001,
  ///   0b10000000, 0b00000001,
  ///   0b11111111, 0b11111111,
  /// ];
  /// printer.print_bitmap(width = 16, height = 8, w_bytes = 2, &bitmap);
  /// ```
  pub fn print_bitmap(&mut self, width: u16, height: u16, w_bytes: usize, bitmap: &[u8]) {
    let flush_height: u16 = 64;
    let mut cmd: Vec<u8> = Vec::with_capacity(4 + (w_bytes * flush_height as usize));
    // self.print_bytes(&[GS, 0x76, 0x30, 0x00]);
    // if width > 382 { return };
    // self.print_bytes(self.to_two_byte(w_bytes as u16).as_ref());
    // self.print_bytes(self.to_two_byte(height as u16).as_ref());

    let mut last_pos: usize = 0;
    let mut last_height: u16 = 0;
    loop {
      let range_end = (last_pos + (w_bytes * flush_height as usize)).clamp(0, bitmap.len());
      let next_height = (last_height + flush_height).clamp(0, height as u16);
      let part_height: u16 = next_height - last_height;

      cmd.extend_from_slice(&[GS, 0x76, 0x30, 0x00]);
      cmd.extend_from_slice(&self.to_two_byte(w_bytes as u16));
      cmd.extend_from_slice(&self.to_two_byte(part_height));
      cmd.extend_from_slice(&bitmap[last_pos..range_end]);

      self.write_vec(&cmd);
      self.flush_buf();
      self.print_bytes(&[0x0c]);
      cmd.clear();

      // self.print_bytes(&bitmap[last_pos..range_end]);
      last_height += 32;
      last_pos = range_end;
      if range_end == bitmap.len() {
        break
      }
      std::thread::sleep(std::time::Duration::from_millis(1500));
    }
    // cmd.extend_from_slice(bitmap);
    // cmd.extend_from_slice("\r\n".as_bytes());

    #[cfg(debug_assertions)]
    for i in 0..height {
      for j in 0..w_bytes {
        let pos = j + (i as usize * w_bytes);
        // print!(" {}: ", pos);
        // let byte = &bitmap[pos];
        print!("{:08b}", &bitmap[pos]);
        // cmd.push(byte);
      }
      print!("\n");
    }

    // cmd.append(&mut Vec::from(bitmap));
    // self.write_vec(&cmd);
    // self.flush_buf();

    #[cfg(debug_assertions)]
    println!("dimensions: {:?}x{:?}", width, height);
  }

  /// # About
  /// Takes in the path to an image file, scales the image to the width provided
  /// and turns it into a black & white image.
  ///
  /// Uses the Floyd-Steinberg dithering algorithm as described on:
  ///
  /// <https://en.wikipedia.org/wiki/Floyd%E2%80%93Steinberg_dithering>
  ///
  /// # Panics
  /// - if the file cannot be found
  pub fn print_image(&mut self, path: &str, width:u32) {
    fn get_pixel(vector: &Vec<Vec<u8>>,x: i32, y: i32) -> u8 {
      if x >= 0 && x < vector.len() as i32 && y >= 0 && y < vector.get(0).unwrap().len() as i32 {
        if let Some(row) = vector.get(x as usize) {
          if let Some(pixel) = row.get(y as usize) {
            return *pixel;
          } else {
            panic!();
          }
        } else {
          panic!();
        }
      } else {
        return 0;
        // panic!("panicked while accessing coords: {:?},{:?}", x, y);
      }
    }

    fn set_pixel(vector: &mut Vec<Vec<u8>>,x: i32, y: i32, val: u8) {
      if x >= 0 && x < vector.len() as i32 && y >= 0 && y < vector.get(0).unwrap().len() as i32 {
        if let Some(row) = vector.get_mut(x as usize) {
          if let Some(pixel) = row.get_mut(y as usize) {
            *pixel = val;
          }
        }
      }
    }

    fn add_error(vector: &mut Vec<Vec<u8>>,x: i32, y: i32, val: &i32, importance: i32) {
      let error: i32 = (*val as f32 * (importance as f32 / 16.0)).round() as i32;
      if x >= 0 && x < vector.len() as i32 && y >= 0 && y < vector.get(0).unwrap().len() as i32 {
        if let Some(row) = vector.get_mut(x as usize) {
          if let Some(pixel) = row.get_mut(y as usize) {
            *pixel = (*pixel as i32 + error).clamp(0, 255) as u8;
          }
        }
      }
    }

    let mut img = match image::open(path) {
      Ok(o) => o,
      Err(e) => panic!("error opening image: {}", e)
    };
    let height: u32 = (img.height() as f32 * (width as f32/ img.width() as f32)) as u32;
    img = img.resize(width, height, imageops::Triangle);
    img.adjust_contrast(-90.0);
    let mut alphaimg = img.to_rgba32f();
    let mut img: image::ImageBuffer<Luma<u8>, Vec<u8>> = image::ImageBuffer::new(img.width(), img.height());
    for pix in alphaimg.enumerate_pixels_mut() {
      let mut max: f32 = 0.0;
      let mut min: f32 = 1.0;
      for channel in 0..=2 {
        pix.2.channels_mut()[channel] = pix.2.channels()[channel] * pix.2.channels()[3] + (1.0 * (1.0 - pix.2.channels()[3]));
        if pix.2.channels()[channel] < min {
          min = pix.2.channels()[channel];
        }
        if pix.2.channels()[channel] > max {
          max = pix.2.channels()[channel];
        }
      }
      let lightness: u8 = (((max + min) / 2.0) * 255.0).round() as u8;
      img.put_pixel(pix.0, pix.1, Luma([lightness]));
    }

    let mut dithered_img = image::GrayImage::new(width + 1, height + 1);

    let mut grayscale = vec![vec![0u8 ; height as usize]; width as usize];
    // let mut bitmap = vec![vec![0u8 ; (width as f32 / 8.0).ceil() as usize]; height as usize];
    let mut bitmap = BitImage::new(width as usize, height as usize);

    for pix in img.enumerate_pixels() {
      if let Some(row) = grayscale.get_mut(pix.0 as usize) {
        if let Some(pixel) = row.get_mut(pix.1 as usize) {
          *pixel = pix.2.channels()[0];
        }
      }
    }

    for pos in img.enumerate_pixels() {
      if pos.0 > grayscale.len() as u32 || pos.1 > grayscale.get(0).unwrap().len() as u32 {
        continue;
      }
      let error: i32;
      match get_pixel(&grayscale, pos.0 as i32, pos.1 as i32) {
        x if x> 127 => {
          set_pixel(&mut grayscale, pos.0 as i32, pos.1 as i32, 255);
          dithered_img.put_pixel(pos.0, pos.1, Luma([255]));
          error = x as i32 - 255;
          bitmap.set_pixel(pos.0 as isize, pos.1 as isize, false);
        },
        x => {
          set_pixel(&mut grayscale, pos.0 as i32, pos.1 as i32, 0);
          dithered_img.put_pixel(pos.0, pos.1, Luma([0]));
          error = x as i32;
          bitmap.set_pixel(pos.0 as isize, pos.1 as isize, true);
        }
      };

      add_error(&mut grayscale, pos.0 as i32 + 1, pos.1 as i32, &error, 7);
      add_error(&mut grayscale, pos.0 as i32 - 1, pos.1 as i32 + 1, &error, 3);
      add_error(&mut grayscale, pos.0 as i32, pos.1 as i32 + 1, &error, 5);
      add_error(&mut grayscale, pos.0 as i32 + 1, pos.1 as i32 + 1, &error, 1);
    }

    #[cfg(debug_assertions)]
    match dithered_img.save("D:\\geral\\Caio\\meus_programas\\thermal_printer\\output_dithered.png") {
      Ok(_) => (),
      Err(e) => panic!("error saving: {:?}", e)
    }

    self.print_bitmap(width as u16, height as u16, bitmap.get_width_in_bytes(), bitmap.as_slice());
  }
}

#[cfg(debug_assertions)]
impl Printer {
  pub fn test_bitmap_buffer_size(&mut self) {
    let step_size = 1;
    let mut bitmap: Vec<u8> = Vec::with_capacity(32*256);
    let mut i = 100;
    for k in 0..i*32 {
      bitmap.push((k & 1 & (k/32 & 1)) as u8 * 255);
    }
    let mut input = String::new();
    loop {
      for k in 0..=255 {
        bitmap.push(k & 1 & (i & 1) as u8 * 255);
      }
      println!("Printing 256 X {} bitmap", i);
      self.print_bitmap(256, i, 32, bitmap.as_slice());
      println!("Worked? Y/n");
      std::io::stdin().read_line(&mut input).expect("error: unable to read stdin!");
      match input.trim().to_lowercase().as_str() {
        "y" => {
          i += step_size;
          continue;
        },
        "n" => {
          println!("max size reached! size in bytes: {}", 32 * (i - step_size));
          break;
        },
        _ => {
          i += step_size;
          continue;
        }
      }
    }
  }
}

pub mod examples {
  /// # About
  /// an example 128x64 bitmap
  pub const BITMAP: [u8;1024] = [
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b10000000,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b00000001,0b10000000,0b00000000,0b00001111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111110,0b00000111,0b00110000,0b11110000,0b00000000,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111100,0b00011100,0b11111100,0b00000000,0b00000000,0b00001111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111000,0b01100111,0b11111111,0b00000000,0b00111110,0b00000111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110000,0b11011111,0b00000000,0b10000000,0b11100001,0b10000011,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110000,0b00111100,0b00000000,0b10000000,0b00000000,0b11000001,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11100000,0b00011000,0b00010000,0b00000000,0b10000000,0b01000001,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11100000,0b00000111,0b11111110,0b00000000,0b10000000,0b00000001,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11000000,0b00001111,0b11110111,0b10000000,0b00111111,0b10000001,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110000,0b11011111,0b11110001,0b11000000,0b01111111,0b11011000,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11100000,0b11011111,0b11111110,0b11000011,0b11111111,0b11011111,0b00111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b00011111,0b00000000,0b01001111,0b11000011,0b11111000,0b00000001,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b01111111,0b11100001,0b11000001,0b00000000,0b11000000,0b00011101,0b11011111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111110,0b01100000,0b11111111,0b10000000,0b00000000,0b10000010,0b01111111,0b10111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111010,0b11000110,0b00011110,0b00000000,0b00000000,0b11000011,0b11100011,0b10111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111010,0b11000111,0b10000000,0b00000001,0b10000000,0b11100001,0b11001000,0b00111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111010,0b10111101,0b11100000,0b01111111,0b10000000,0b00111000,0b00011000,0b10111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111010,0b11111100,0b01111100,0b01110110,0b01100000,0b00111100,0b00011100,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111110,0b11000110,0b00011111,0b00000110,0b11111000,0b01111111,0b00011100,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111101,0b01000111,0b00011011,0b11110011,0b00000010,0b11100000,0b00111101,0b10111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111110,0b10000011,0b11101000,0b11111111,0b00000011,0b11000000,0b11111100,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b10000011,0b11111100,0b00001111,0b11111101,0b00001111,0b11111100,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b10000001,0b10111111,0b10001101,0b11111111,0b11111111,0b01101100,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b10000000,0b11001111,0b11111000,0b00011011,0b11110011,0b01101100,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11000000,0b01101011,0b11111100,0b00011000,0b11000011,0b01111100,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11100000,0b01111000,0b01111111,0b11111111,0b11111111,0b11111100,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11100000,0b00111000,0b00011111,0b11111111,0b11111111,0b11111100,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110000,0b00011100,0b00011001,0b11111111,0b11111111,0b11111100,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111000,0b00001111,0b00110000,0b00111111,0b11111111,0b11111100,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111100,0b00000011,0b10110000,0b00110000,0b11111111,0b10111100,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111110,0b00000001,0b11110000,0b00110000,0b11001101,0b10111000,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b01100100,0b01111110,0b00110000,0b10011001,0b11110000,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b10011111,0b00001111,0b11111111,0b11111111,0b11100000,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11100111,0b11100000,0b11111111,0b11111110,0b00000000,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111001,0b11111000,0b00000000,0b00000000,0b00110000,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111110,0b00011110,0b00000000,0b00000000,0b11100010,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b10001111,0b11111111,0b11111111,0b10000100,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11100000,0b11111111,0b11000000,0b00011000,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111000,0b00000111,0b11111111,0b11110000,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b00000000,0b00000000,0b00000000,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110000,0b00000000,0b00000001,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b00000000,0b00000011,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111110,0b00111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11110111,0b11111111,0b01111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b00000000,0b00111111,0b11111111,0b11111111,0b11000111,0b11111000,0b01111111,0b11111111,0b11111111,0b11111111,0b10000111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11000111,0b00011111,0b11111111,0b11111111,0b00000111,0b11100000,0b11111111,0b11111111,0b11111111,0b11111111,0b00000011,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b11000111,0b10001111,0b11111111,0b11111111,0b10001111,0b11111000,0b11111111,0b11111111,0b11111111,0b11111111,0b01100011,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b10001111,0b10001111,0b11111111,0b11111111,0b10001111,0b11111000,0b11111111,0b11111111,0b11111111,0b11111110,0b11100011,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b10001111,0b10001111,0b11111111,0b11111111,0b10001111,0b11110001,0b11111111,0b11111111,0b11111111,0b11111111,0b11100111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b10001111,0b10011100,0b11001111,0b00001111,0b00011001,0b11110001,0b11111000,0b01111100,0b11100111,0b10001111,0b11000111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b10001111,0b00010001,0b10011100,0b11000111,0b00010000,0b11110001,0b11100110,0b00110000,0b10000110,0b00001111,0b10001111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b00001110,0b00100001,0b00011001,0b11000111,0b00001000,0b11110011,0b11001110,0b00100001,0b01000101,0b00011111,0b00111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b00000000,0b11010000,0b01110001,0b11000111,0b00011000,0b11100011,0b10001100,0b01010000,0b01000011,0b00011111,0b01111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111111,0b00011111,0b11110010,0b11110001,0b11000110,0b00111000,0b11100011,0b10011000,0b11110000,0b11000011,0b00011110,0b01111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111110,0b00111111,0b11110001,0b11100011,0b11000110,0b00111000,0b11100011,0b00000111,0b11110001,0b11000110,0b00111110,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111110,0b00111111,0b11100001,0b11100011,0b10001110,0b01111001,0b11000111,0b00011111,0b11100001,0b10000110,0b00111111,0b11111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111110,0b00111111,0b11100011,0b11100011,0b10001100,0b01110001,0b11000111,0b00011111,0b11100011,0b10001110,0b00101100,0b01111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111100,0b01111111,0b11100011,0b11100011,0b10011100,0b01110011,0b11000101,0b00011100,0b11100011,0b10001100,0b00011000,0b01111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11111100,0b01111111,0b11000111,0b11100011,0b00111100,0b01100111,0b10000011,0b00000001,0b11000011,0b00001100,0b00111000,0b01111111,0b11111111,0b11111111,
  0b11111111,0b11111111,0b11100000,0b00001111,0b11000111,0b11110000,0b11111110,0b00011111,0b10001111,0b10000111,0b11000111,0b00011100,0b11111000,0b11111111,0b11111111,0b11111111,
  ];
}
