#![allow(dead_code, unused_variables)]

pub mod printing;
pub mod bitimage;

use core::panic;
use std::{env, path::Path};
use clap::{Parser, Arg};

fn main() {
  env::set_var("RUST_BACKTRACE", "1");

  let mut cmd = clap::Command::new("thermal_printer")
    .arg(Arg::new("path_to_printer")
      .short('p')
      .long("path")
      .required(true)
      .help("path to the printer file handle, see documentation for help (todo)")
    )
    .arg(Arg::new("input")
      .short('i')
      .long("input")
      .help("image file path")
    )
    .arg(Arg::new("width")
      .short('w')
      .long("width")
      .default_value("128")
      .help("width of the image in pixels")
    )
    .arg(Arg::new("qr_code_width")
      .long("qr_width")
      .help("width of the qr code, must be in range 0..16")
    )
    .arg(Arg::new("qr_code")
      .short('q')
      .long("qr_code")
      .help("print a qr code with the given text encoded into it")
    )
    .arg(Arg::new("debug")
     .short('d')
     .long("debug")
     .takes_value(false)
    )
    .arg(Arg::new("text")
      .short('t')
      .long("text")
      .help("print the given text")
    )
    .arg(Arg::new("justification")
      .short('j')
      .long("justification")
      .default_value("left")
      .help("must be either \"left\", \"center\" or \"right\", falls back to \"left\"")
    )
  ;
  let cur_dir = env::current_dir().expect("error getting cwd!").to_str().expect("error turning path into string!").to_owned();

  let args = cmd.clone().get_matches();
  let mut printer = printing::Printer::new(args.value_of("path_to_printer").unwrap());

  match args.value_of("justification").unwrap().to_lowercase().as_str() {
    "left" => printer.set_justification(0),
    "center" => printer.set_justification(1),
    "right" => printer.set_justification(2),
    _ => printer.set_justification(0)
  }

  if let Some(path) = args.value_of("input") {
    let image_path: String;

    if Path::new(&path).exists() {
      image_path = path.to_owned();
    } else if Path::new(&(cur_dir.clone() + &path)).exists() {
      image_path = cur_dir + &path
    } else {
      panic!("error finding file!");
    }
    printer.print_image(image_path.as_str(), args.value_of("width").unwrap().parse().unwrap());

    return
  }

  if let Some(qr_code_text) = args.value_of("qr_code") {
    printer.print_qr_code(args.value_of("qr_code_width").unwrap().parse().unwrap(), qr_code_text.as_bytes());
    if args.is_present("debug") {
      printer.println(qr_code_text);
    }
    return
  }

  if let Some(text) = args.value_of("text") {
    printer.println(text);
    return
  }

  cmd.print_help().expect("failed printing help!");

  // printer.print_qr_code(10, b"https://oisumida.rs/");

  // printer.print_bitmap(128, 64, 16, &printing::examples::BITMAP);
  }
