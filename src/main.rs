use core::panic;
use std::{fs::File, io::Write, path::Path};
use image::{imageops, Luma, Pixel, ImageBuffer};
use bitvec::{view::BitView, ptr::{BitRef, Const}};

struct Printer {
  path: String,
  file_handle: File,
}

impl Printer {
  fn new(printer_path: &str) -> Self {
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

  fn to_two_byte(&self, num: u16) -> [u8;2] {
    let mut bytes: [u8;2] = num.to_be_bytes();
    bytes.reverse();
    bytes
  }

  fn println(&mut self, message: &str) {
    match self.file_handle.write_all(message.as_bytes()) {
      Ok(_) => (),
      Err(e) => panic!("error: {}", e)
    };
    self.write_byte(0x0c);
    match self.file_handle.flush() {
      Ok(_) => println!("printed {:?} to: {}", message, self.path),
      Err(e) => panic!("error: {}", e)
    };
  }

  fn print_bytes(&mut self, message: &[u8]) {
    match self.file_handle.write_all(message) {
      Ok(_) => (),
      Err(e) => panic!("error: {}", e)
    };
    match self.file_handle.flush() {
      Ok(_) => println!("printed {:?} to: {}", message, self.path),
      Err(e) => panic!("error: {}", e)
    };
  }

  fn write_vec(&mut self, bytes: &Vec<u8>) {
    for byte in bytes {
      self.write_byte(*byte);
    }
  }

  fn print_bitmap(&mut self, width:u32, height: u32, w_bytes: usize, bitmap: &[u8]) {
    let mut cmd: Vec<u8> = Vec::from([0x1d, 0x76, 0x30, 0x03]);
    if width > 382 { return };
    cmd.extend_from_slice(self.to_two_byte(w_bytes as u16).as_ref());
    cmd.extend_from_slice(self.to_two_byte(height as u16).as_ref());
    cmd.extend_from_slice(bitmap);
    cmd.extend_from_slice("\r\n".as_bytes());
    self.write_vec(&cmd);
    self.flush_buf();
    for i in 0..height {
      for j in 0..width/8 {
        let byte = cmd[((i+1) * j) as usize];
        print!("{:08b}", byte);
      }
      print!("\n");
    }
    println!("dimensions: {:?}x{:?}", width, height);
  }

  fn print_image(&mut self, path: &str, width:u32) {

    let img = match image::open(path) {
      Ok(o) => o,
      Err(e) => panic!("error opening image: {}", e)
    };
    let height: u32 = (img.height() as f32 * (width as f32/ img.width() as f32)) as u32;
    imageops::resize(&img, width, height, imageops::Triangle);
    imageops::grayscale(&img);
    let img = img.to_luma8();
    let mut dithered_img = image::GrayImage::new(width + 1, height + 1);

    let mut grayscale = vec![vec![0u8 ; height as usize]; width as usize];
    let mut bitmap = vec![vec![0u8 ; height as usize]; (width as f32 / 8.0).ceil() as usize];

    for pix in img.enumerate_pixels() {
      if let Some(row) = grayscale.get_mut(pix.0 as usize) {
        if let Some(pixel) = row.get_mut(pix.1 as usize) {
          *pixel = pix.2.channels()[0];
        }
      }
    }

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

    fn add_bit_to_bitmap(vector: &mut Vec<Vec<u8>>,x: i32, y: i32, val: bool) {
      if x >= 0 && x < vector.len() as i32 * 8 && y >= 0 && y < vector.get(0).unwrap().len() as i32 {
        if let Some(row) = vector.get_mut((x as f32 / 8.0).floor() as usize) {
          if let Some(pixel) = row.get_mut(y as usize) {
            if val {
              *pixel = (*pixel << 1) + 1;
            } else {
              *pixel = *pixel << 1;
            }
          }
        }
      }
    }

    fn add_error(vector: &mut Vec<Vec<u8>>,x: i32, y: i32, val: &i32, importance: i32) {
      let error: i32 = (*val as f32/ importance as f32).round() as i32;
      if x >= 0 && x < vector.len() as i32 && y >= 0 && y < vector.get(0).unwrap().len() as i32 {
        if let Some(row) = vector.get_mut(x as usize) {
          if let Some(pixel) = row.get_mut(y as usize) {
            *pixel = (*pixel as i32 + error).clamp(0, 255) as u8;
          }
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
          add_bit_to_bitmap(&mut bitmap, pos.0 as i32, pos.1 as i32, false);
        },
        x => {
          set_pixel(&mut grayscale, pos.0 as i32, pos.1 as i32, 0);
          dithered_img.put_pixel(pos.0, pos.1, Luma([0]));
          error = x as i32;
          add_bit_to_bitmap(&mut bitmap, pos.0 as i32, pos.1 as i32, true);
        }
      };

      add_error(&mut grayscale, pos.0 as i32 + 1, pos.0 as i32, &error, 7);
      add_error(&mut grayscale, pos.0 as i32 - 1, pos.0 as i32+ 1,&error, 3);
      add_error(&mut grayscale, pos.0 as i32, pos.0 as i32 + 1, &error, 5);
      add_error(&mut grayscale, pos.0 as i32 + 1, pos.0 as i32 + 1, &error, 1);
    }

    match dithered_img.save("D:\\geral\\Caio\\meus_programas\\thermal_printer\\output.png") {
      Ok(_) => (),
      Err(e) => panic!("error saving: {:?}", e)
    }

    println!("{:?}", &bitmap.concat());
    // self.print_bitmap(width, height, bitmap.len(), &bitmap.concat());
  }
}

fn main() {
  let printer_location = "\\\\DESKTOP-GPDFQCL\\Thermal_Printer";
  let message = "BRUH!\n";
  let mut printer = Printer::new(printer_location);
  // printer.println(message);
  // printer.flush_buf();
  // printer.print_bytes(&[0x1d, 0x76, 0x30, 0x00, 0x01, 0x00, 0x80, 10]);
  let bitmap: [u8;1024] = [
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

  printer.print_image("D:/geral/Caio/meus_programas/thermal_printer/sample.webp", 128);

  // printer.print_bitmap(16, 64, &bitmap);
  // printer.print_bitmap(1, 1, &[
    //   0b11111111,
    //   0b11111111,
    //   0b11111111,
    //   0b11111111,
    //   0b11111111,
    //   0b11111111,
    //   0b11111111,
    //   0b11111111,
    // ]);
    // printer.flush_buf();
  }
