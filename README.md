# Windows Compatible ESC-POS Thermal Printer Control
## About
This program allows you to use a USB thermal printer to print text, images, barcodes and QR codes.

Made entirely using Rust!
## Features
### Checklist:
- [x] Print text.
- [x] Print images using the Floyd-Steinberg error diffusion dithering algorithm, preserving their details.
- [x] Image scaling.
- [x] Print QR Codes.
- [ ] Print Bar Codes.
- [x] Set justification for text, images, and QR Codes.
- [ ] Change text settings to make it big, bold, italics, etc.

### Maybe one day:
- Graphing
- Custom fonts
- Support for other platforms
- Markdown support
## Usage
### On windows:
- Connect your thermal printer to your windows computer using the generic text-only printer driver.
- Enable Network Sharing on your printer's properties window. The name you choose on the sharing menu will be the one you input into the program.
- Download the program and compile it using ``cargo build --release`` or download the compiled binaries.
- Open a command prompt on the executable's folder and do ``thermal_printer.exe -p [PRINTER'S NETWORK NAME]``. If no errors are reported, then the printer  has been correctly set up.
- Do ``thermal_printer.exe --help`` to learn about the arguments and their usages.
