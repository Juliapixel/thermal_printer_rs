pub struct BitImage {
  bytes: Vec<u8>,
  width: usize,
  height: usize,
  w_bytes: usize
}

impl BitImage {
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

  pub fn get_width_in_bytes(&self) -> usize {
    self.w_bytes
  }

  fn is_within_bounds(&self, x: isize, y: isize) {
    if x < 0 && x >= self.width as isize && y < 0 && y >= self.height as isize {
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
