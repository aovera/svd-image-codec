mod encoder;
mod decoder;

use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use minifb::{Window, WindowOptions};
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;

fn main() {
    let args : Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "encode" => handle_encode(&args),
        "view" => handle_view(&args),
        _ => {
            println!("Error: Unknown command '{}'", command);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("SVD image codec");
    println!("\tTo encode: svd encode [-k RANK_PERCENTAGE] INPUT OUTPUT");
    println!("\tTo view: svd view SVD_FILE");
    println!("\nUsage expamles:");
    println!("\tsvd encode input.png output.svd");
    println!("\tsvd encode -k 20 input.png output.svd");
    println!("\tsvd view output.svd");
}

fn handle_encode(args : &[String]) {
    // Parse arguments
    let mut rank_ratio = 0.05; // Default 5%
    let mut input_idx = 2;

    if args.len() < 4 {
        print_usage();
        return;
    }

    if args[2] == "-k" {
        if args.len() < 6 {
            print_usage();
            return;
        }
        let k_val: f32 = args[3].parse().unwrap_or_else(|_| {
            println!("Error: Rank ratio is invalid.");
            std::process::exit(1);
        });
        rank_ratio = k_val / 100.0;
        input_idx = 4;
    }

    let input_path = &args[input_idx];
    let output_path = &args[input_idx + 1];

    println!("Reading image: {}", input_path);
    let img = image::open(Path::new(input_path)).unwrap_or_else(|e| {
        panic!("Could not open image: {}", e);
    }).to_rgb8();

    let (width, height) = img.dimensions();
    let mut raw_pixels = vec![0u32; (width * height) as usize];

    // COnvert RGB8 to linear u32 format
    for (i, pixel) in img.pixels().enumerate() {
        let r = pixel[0] as u32;
        let g = pixel[1] as u32;
        let b = pixel[2] as u32;
        raw_pixels[i] = (r << 16) | (g << 8) | b;
    }

    println!("[Encoder] Calculating SVD (Rank ratio: %{})...", rank_ratio * 100.0);
    let encoded = encoder::encode_image(&raw_pixels, width, height, rank_ratio);

    // Serialize and write to disk
    println!("[I/O] Saving: {}", output_path);
    let encoded_bytes = bincode::serialize(&encoded).unwrap();
    let file = File::create(output_path).unwrap();
    let mut compressor = ZlibEncoder::new(file, Compression::best());
    compressor.write_all(&encoded_bytes).unwrap();
    compressor.finish().unwrap();
    println!("Done.");
}

fn handle_view(args : &[String]) {
    if args.len() < 3 {
        print_usage();
        return;
    }

    let input_path = &args[2];
    println!("Reading: {}", input_path);

    // Read and deserialize
    let file = File::open(input_path).unwrap_or_else(|_| {
        panic!("Could not open: {}", input_path);
    });
    let mut decompressor = ZlibDecoder::new(file);
    let mut buffer = Vec::new();
    decompressor.read_to_end(&mut buffer).unwrap();

    let encoded : encoder::EncodedImage = bincode::deserialize(&buffer).unwrap();
    let width = encoded.width;
    let height = encoded.height;

    println!("[Decoder] Rebuilding matrices...");
    let decoded_pixels = decoder::decode_image(&encoded);

    // Print with minifb
    let mut window = Window::new(
        &format!("SVD Viewer - {}", input_path),
        width as usize,
        height as usize,
        WindowOptions::default(),
    )
    .unwrap();

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    println!("[Viewer] Tap ESC to exit.");

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        window
            .update_with_buffer(&decoded_pixels, width as usize, height as usize)
            .unwrap();
    }
}
