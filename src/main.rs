#![allow(dead_code, unused_variables)]

pub mod printing;
pub mod bitimage;

use std::{env, path::PathBuf};
use clap::Arg;

fn main() {
  #[cfg(debug_assertions)]
  env::set_var("RUST_BACKTRACE", "1");

  let cmd = clap::Command::new("thermal_printer")
    .arg(Arg::new("path_to_printer")
      .short('p')
      .long("path")
      .required(true)
      .takes_value(true)
      .help("path to the printer file handle, see documentation for help (todo)")
    )
    .arg(Arg::new("input")
      .short('i')
      .long("input")
      .takes_value(true)
      .value_parser(clap::value_parser!(PathBuf))
      .help("image file path")
    )
    .arg(Arg::new("width")
      .short('w')
      .long("width")
      .takes_value(true)
      .default_value("128")
      .help("width of the image in pixels")
    )
    .arg(Arg::new("qr_code_width")
      .long("qr_width")
      .takes_value(true)
      .default_value("5")
      .help("width of the qr code, must be in range 0..16")
    )
    .arg(Arg::new("qr_code")
      .short('q')
      .long("qr_code")
      .takes_value(true)
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
      .takes_value(true)
      .help("print the given text")
    )
    .arg(Arg::new("justification")
      .short('j')
      .long("justification")
      .default_value("left")
      .help("must be either \"left\", \"center\" or \"right\", falls back to \"left\"")
    ).get_matches()
  ;

  let cur_dir = env::current_dir().expect("error getting cwd!").to_str().expect("error turning path into string!").to_owned();

  let mut printer = printing::Printer::new(cmd.get_one::<String>("path_to_printer").expect("path argument invalid!"));

  match cmd.get_one::<String>("justification").unwrap().to_lowercase().as_str() {
    "left" => printer.set_justification(0),
    "center" => printer.set_justification(1),
    "right" => printer.set_justification(2),
    _ => printer.set_justification(0)
  }

  if let Some(path) = cmd.get_one::<PathBuf>("input") {
    let image_path: &str;

    // if Path::new(&path).exists() {
    //   image_path = path.to_owned();
    // } else if Path::new(&(cur_dir.clone() + &path)).exists() {
    //   image_path = cur_dir + &path
    // } else {
    //   panic!("error finding file!");
    // }
    if path.exists() {
      image_path = path.to_str().expect("error parsing image path!");
      printer.print_image(image_path, cmd.get_one::<String>("width").expect("error parsing image width!").parse().expect("error parsing image width!"));
    }
    return
  }

  if let Some(qr_code_text) = cmd.get_one::<String>("qr_code") {
    printer.print_qr_code(cmd.get_one::<String>("qr_code_width").expect("error parsing qr code width!").parse().expect("qr code width not a number!"), qr_code_text.as_bytes());
    if cmd.contains_id("debug") {
      printer.println(qr_code_text);
    }
    return
  }

  if let Some(text) = cmd.get_one::<String>("text") {
    printer.println(text);
    return
  }

  // printer.print_qr_code(10, b"https://oisumida.rs/");

  // printer.print_bitmap(128, 64, 16, &printing::examples::BITMAP);
  }
