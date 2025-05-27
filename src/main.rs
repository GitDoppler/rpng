use encoder::{CompressionMethod, save_to_png_with_compression};
use image::ImageReader;
use std::env;
use std::path::{Path, PathBuf};

mod encoder;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    let mut compression_method = CompressionMethod::Custom;
    let mut image_path = &args[1];
    let mut output_path_arg = args.get(2);

    if args.len() >= 2 && (args[1] == "--custom" || args[1] == "--flate2") {
        if args.len() < 3 {
            print_usage(&args[0]);
            std::process::exit(1);
        }

        compression_method = match args[1].as_str() {
            "--custom" => CompressionMethod::Custom,
            "--flate2" => CompressionMethod::Flate2,
            _ => CompressionMethod::Custom,
        };

        image_path = &args[2];
        output_path_arg = args.get(3);
    }

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

    let output_path = if let Some(path_str) = output_path_arg {
        let mut path = PathBuf::from(path_str);
        if path.extension().map_or(true, |ext| ext != "png") {
            path.set_extension("png");
        }
        path
    } else {
        let input_path = Path::new(image_path);
        get_output_path(input_path)
    };

    match save_to_png_with_compression(&image, &output_path.to_string_lossy(), compression_method) {
        Ok(_) => {
            let method_name = match compression_method {
                CompressionMethod::Custom => "custom DEFLATE",
                CompressionMethod::Flate2 => "flate2 DEFLATE",
            };
            println!(
                "Successfully converted to PNG using {}: {}",
                method_name,
                output_path.display()
            );
        }
        Err(e) => {
            eprintln!("Error saving image: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_usage(program_name: &str) {
    eprintln!("Usage:");
    eprintln!(
        "  {} [--custom|--flate2] <image_path> [output_path]",
        program_name
    );
    eprintln!();
    eprintln!("Compression Methods:");
    eprintln!("  --custom  Use our custom simplified DEFLATE algorithm (default)");
    eprintln!("  --flate2  Use the standard flate2 DEFLATE implementation");
    eprintln!();
    eprintln!("Examples:");
    eprintln!(
        "  {} photo.jpg                    # Use custom compression",
        program_name
    );
    eprintln!("  {} --custom photo.jpg output.png", program_name);
    eprintln!("  {} --flate2 photo.jpg output.png", program_name);
}

fn get_output_path(input_path: &Path) -> PathBuf {
    let stem = input_path.file_stem().unwrap_or_default();
    let parent = input_path.parent().unwrap_or_else(|| Path::new(""));

    let mut output_path = PathBuf::from(parent);
    output_path.push(stem);
    output_path.set_extension("png");

    output_path
}
