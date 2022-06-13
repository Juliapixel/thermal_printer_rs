#![allow(dead_code, unused_variables)]

pub mod printing;
pub mod bitimage;

use core::panic;
use std::{env, path::Path};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
  #[clap(short, long, help = "path to printer device")]
  path_to_printer: String,

  #[clap(short, long, help = "image file input")]
  input: Option<String>,

  #[clap(short, long, default_value_t = 128, help = "width in dots for images")]
  width: u32,

  #[clap(long, default_value_t = 7, help = "width used for qr codes, must be in range 0..16")]
  qr_code_width: u8,

  #[clap(short, long)]
  qr_code: Option<String>,

  #[clap(short, long)]
  debug: bool,

  #[clap(short, long)]
  text: Option<String>,

  #[clap(short, long, help = "must be either \"left\", \"center\" or \"right\", fallsback to \"left\"", default_value = "left")]
  justification: String
}

fn main() {
  env::set_var("RUST_BACKTRACE", "1");


  let cur_dir = env::current_dir().expect("error getting cwd!").to_str().expect("error turning path into string!").to_owned();

  let args = Args::parse();
  let mut printer = printing::Printer::new(args.path_to_printer.as_str());

  match args.justification.to_lowercase().as_str() {
    "left" => printer.set_justification(0),
    "center" => printer.set_justification(1),
    "right" => printer.set_justification(2),
    _ => printer.set_justification(0)
  }

  if let Some(path) = args.input {
    let image_path: String;

    if Path::new(&path).exists() {
      image_path = path;
    } else if Path::new(&(cur_dir.clone() + &path)).exists() {
      image_path = cur_dir + &path
    } else {
      panic!("error finding file!");
    }
    printer.print_image(image_path.as_str(), args.width);

    return
  }

  if let Some(qr_code_text) = args.qr_code {
    printer.print_qr_code(args.qr_code_width, qr_code_text.as_bytes());
    if args.debug {
      printer.println(qr_code_text.as_str());
    }
    return
  }

  if let Some(text) = args.text {
    printer.println(text.as_str());
    return
  }

  // printer.print_qr_code(10, b"https://oisumida.rs/");

  // printer.print_bitmap(128, 64, 16, &printing::examples::BITMAP);
  }
