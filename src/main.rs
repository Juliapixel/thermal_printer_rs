#![allow(dead_code, unused_variables)]

pub mod printing;
pub mod bitimage;

use std::{env, path::{PathBuf, Path}, fs::File, io::{BufReader}};
use clap::Arg;

fn main() {
  #[cfg(debug_assertions)]
  env::set_var("RUST_BACKTRACE", "1");

  let mut cmd = clap::Command::new("thermal_printer")
    .arg(Arg::new("path_to_printer")
      .short('p')
      .long("path")
      .required(true)
      .takes_value(true)
      .help("path to the printer file handle, see documentation for help")
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
      .help("width of the image in dots")
    )
    .arg(Arg::new("qr_code_width")
      .long("qr_width")
      .takes_value(true)
      .default_value("8")
      .help("width of the qr code, must be in range 0..16")
    )
    .arg(Arg::new("qr_code")
      .short('q')
      .long("qr_code")
      .takes_value(true)
      .help("print a qr code with the given text encoded into it")
    )
    .arg(Arg::new("text")
      .short('t')
      .long("text")
      .takes_value(true)
      .help("print the given text")
    )
    .arg(Arg::new("markdown")
    .long("md")
    .takes_value(true)
    .value_parser(clap::value_parser!(PathBuf))
    .help("print the given markdown file")
  )
    .arg(Arg::new("justification")
      .short('j')
      .long("justification")
      .takes_value(true)
      .default_value("left")
      .help("must be either \"left\", \"center\" or \"right\", falls back to \"left\"")
    )
    .arg(Arg::new("reset")
    .long("reset")
    .takes_value(false)
    .help("resets the printer back to its initial state\nmust be used alone")
    )
    .arg(Arg::new("dithering")
    .long("dithering")
    .takes_value(true)
    .default_value("2sierra")
    .help("select the dithering mode used to print images\navailable modes: sierra, 2sierra, fs, none\nfalls back to 2sierra")
    )
  ;
  #[cfg(debug_assertions)]
  {
    cmd = cmd.arg(Arg::new("test_buffer_size")
      .long("test_buffer_size")
      .takes_value(false)
      .help("tests the max bitmap buffer size supported by the printer, in chunks of 32 bytes")
      )
      .arg(Arg::new("debug")
      .short('d')
      .long("debug")
      .takes_value(false)
      .help("used in development to send specific commands to the printer\nonly use if you know what you're doing")
      )
    ;
  }
  let args = cmd.get_matches();

  let cur_dir = env::current_dir().expect("error getting cwd!").to_str().expect("error turning path into string!").to_owned();

  let printer_path = {
    let path_arg = args.get_one::<String>("path_to_printer").expect("path argument invalid!");
    String::from("\\\\127.0.0.1\\") + path_arg
  };
  let mut printer = printing::Printer::new(&printer_path);

  match args.get_one::<String>("justification").unwrap().to_lowercase().as_str() {
    "left" => printer.set_justification(0),
    "center" => printer.set_justification(1),
    "right" => printer.set_justification(2),
    _ => printer.set_justification(0)
  }

  #[cfg(debug_assertions)]
  {
    if args.contains_id("test_buffer_size") {
      printer.test_bitmap_buffer_size();
      return
    }

    if args.contains_id("debug") {
      todo!()
    }
  }

  if args.contains_id("reset") {
    printer.reset();
    return
  }

  if let Some(path) = args.get_one::<PathBuf>("markdown") {
    if path.to_str().unwrap().ends_with(".md") {
      let md_file = File::open(path).unwrap();
      let md_lines = BufReader::new(md_file);
      printer.print_markdown(md_lines);
    }
  }

  if let Some(path) = args.get_one::<PathBuf>("input") {
    let image_path: &str;
    let dithering: u8 = match args.get_one::<String>("dithering").unwrap().to_lowercase().as_str() {
      "sierra" => 2,
      "fs" => 0,
      "none" => 255,
      _ => 1
    };
    if path.exists() {
      image_path = path.to_str().expect("error parsing image path!");
      printer.print_image(image_path, args.get_one::<String>("width").expect("error parsing image width!").parse().expect("error parsing image width!"), dithering);
    }
    return
  }

  if let Some(qr_code_text) = args.get_one::<String>("qr_code") {
    printer.print_qr_code(args.get_one::<String>("qr_code_width").expect("error parsing qr code width!").parse().expect("qr code width not a number!"), qr_code_text.as_bytes());
    if args.contains_id("debug") {
      printer.println(qr_code_text);
    }
    return
  }

  if let Some(text) = args.get_one::<String>("text") {
    printer.println(text);
    return
  }
}
