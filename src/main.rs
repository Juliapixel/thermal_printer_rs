use std::{fs::File, io::Write, path::Path};

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

  fn print_buffer(&mut self) {
    match self.file_handle.write(&[0x0c]) {
      Ok(_) => (),
      Err(e) => panic!("error: {}", e)
    };
  }

  fn println(&mut self, message: &str) {
    match self.file_handle.write_all(message.as_bytes()) {
      Ok(_) => (),
      Err(e) => panic!("error: {}", e)
    };
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
}

fn main() {
  let printer_location = "\\\\DESKTOP-GPDFQCL\\Thermal_Printer";
  let message = "0x5681235\n";
  let mut printer = Printer::new(printer_location);
  // printer.println(message);
  // printer.print_bytes(&[138, 80, 42, 0x0c])
  printer.print_bytes(&[0x1b, 0x2a, 0, 16, 1]);
  for _ in 0.. {
    printer.write_byte(0b10101010)
  }
  printer.print_buffer();
}
