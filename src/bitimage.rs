/// # About
/// A struct to handle a bitmap image in which each bit represents a pixel
/// # Creating
/// ```
/// let mut bitmap = BitImage::new(128, 64);
/// ```
/// # Usage
/// Setting a pixel
/// ```
/// bitmap.set_pixel(32, 64, true);
/// ```
/// Getting the value of a pixel
/// ```
/// assert_eq!(bitmap.get_pixel(32, 64), true);
/// ```
/// To use with ```Printer::print_bitmap()```
/// ```
/// printer.print_bitmap(bitmap.get_width() as u32, bitmap.get_height() as u32, bitmap.as_slice())
/// ```
pub struct BitImage {
  bytes: Vec<u8>,
  width: usize,
  height: usize,
  w_bytes: usize
}

impl BitImage {
  /// # Examples
  /// ```
  /// let mut bitmap = BitImage::new(128, 64);
  /// ```
  pub fn new(w: usize, h: usize) -> Self {
    let mut bytes_vec = Vec::with_capacity((w as f64 / 8.0).ceil() as usize * h);
    for i in 0..bytes_vec.capacity() {
      bytes_vec.push(0);
    }
    BitImage {
      bytes: bytes_vec,
      width: w,
      height: h,
      w_bytes: (w as f64 / 8.0).ceil() as usize
    }
  }

  pub fn get_width(&self) -> usize {
    self.width
  }

  pub fn get_height(&self) -> usize {
    self.height
  }

  pub fn get_width_in_bytes(&self) -> usize {
    self.w_bytes
  }

  fn is_within_bounds(&self, x: isize, y: isize) {
    if x < 0 || x >= self.width as isize || y < 0 || y >= self.height as isize {
      panic!("tried to read out of bounds at coords: {}, {}", x, y);
    }
  }

  pub fn get_pixel(&self, x: isize, y: isize) -> bool {
    self.is_within_bounds(x, y);
    let position: u8 = 1 << x % 8;
    let pixel_byte: &u8 = match self.bytes.get(x as usize / 8 + (y as usize * self.w_bytes)) {
      Some(o) => o,
      None => panic!("tried to read BitImage pixel that is out of bounds!")
    };
    return *pixel_byte | position == *pixel_byte
  }

  pub fn set_pixel(&mut self, x:isize, y: isize, val: bool) {
    self.is_within_bounds(x, y);
    let position: u8 = 128 >> x % 8;
    let byte_pos = (x as f64 / 8.0).floor() as usize + (y as usize * self.w_bytes);
    let pixel_byte: &mut u8 = match self.bytes.get_mut((x as f64 / 8.0).floor() as usize + (y as usize * self.w_bytes)) {
      Some(o) => o,
      None => panic!("tried to read BitImage pixel that is out of bounds at: {}, {}", x, y)
    };
    if val{
      *pixel_byte |= position;
    } else {
      *pixel_byte = *pixel_byte & !position;
    }
  }

  pub fn as_slice(&self) -> &[u8]{
    self.bytes.as_slice()
  }
}
