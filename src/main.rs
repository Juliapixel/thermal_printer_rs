#![allow(dead_code, unused_variables)]

pub mod printing;
pub mod bitimage;

use core::panic;
use std::env;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
  #[clap(short, long)]
  input: String,

  #[clap(short, long)]
  image_width: u32,

  #[clap(long)]
  qr_code: String,

  #[clap(short, long)]
  text: String,
}

fn main() {
  env::set_var("RUST_BACKTRACE", "1");

  let cur_dir = match env::current_dir() {
    Ok(o) => o,
    Err(e) => panic!("panicked getting cwd! {:?}", e)
  };

  let args = Args::parse();

  let image_location = match image::open(&args.input) {
    Ok(img) => img,
    Err(_) => {
      match image::open(cur_dir.display().to_string() + &args.input) {
        Ok(o) => o,
        Err(_) => panic!()
      }
    }
  };


  let printer_location = "\\\\DESKTOP-GPDFQCL\\Thermal_Printer";
  let message = "BRUH!\n";
  let mut printer = printing::Printer::new(printer_location);

  printer.print_image(args.input.as_str(), 256);

  // printer.print_qr_code(10, b"https://oisumida.rs/");

  // printer.print_bitmap(128, 64, 16, &printing::examples::BITMAP);
  }
