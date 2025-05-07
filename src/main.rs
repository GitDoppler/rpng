use encoder::save_to_png;
use image::ImageReader;
use std::env;
use std::path::{Path, PathBuf};

mod encoder;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <image_path> [output_path]", args[0]);
        std::process::exit(1);
    }

    let image_path = &args[1];

    let image = match ImageReader::open(image_path) {
        Ok(image) => match image.decode() {
            Ok(i) => i,
            Err(e) => {
                eprintln!("Error decoding image: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error opening image: {}", e);
            std::process::exit(1);
        }
    };

    let output_path = if args.len() > 2 {
        let mut path = PathBuf::from(&args[2]);
        if path.extension().map_or(true, |ext| ext != "png") {
            path.set_extension("png");
        }
        path
    } else {
        let input_path = Path::new(image_path);
        get_output_path(input_path)
    };

    // match image.save(&output_path) {
    //     Ok(_) => println!("Successfully converted to PNG: {}", output_path.display()),
    //     Err(e) => {
    //         eprintln!("Error saving image: {}", e);
    //         std::process::exit(1);
    //     }
    // }

    match save_to_png(&image, &output_path.to_string_lossy()) {
        Ok(_) => println!("Successfully converted to PNG: {}", output_path.display()),
        Err(e) => {
            eprintln!("Error saving image: {}", e);
            std::process::exit(1);
        }
    }
}

fn get_output_path(input_path: &Path) -> PathBuf {
    let stem = input_path.file_stem().unwrap_or_default();
    let parent = input_path.parent().unwrap_or_else(|| Path::new(""));

    let mut output_path = PathBuf::from(parent);
    output_path.push(stem);
    output_path.set_extension("png");

    output_path
}
