use clap::Parser;
use exif::{In, Tag};
use image::{imageops, imageops::FilterType, DynamicImage, GenericImage};
use std::cmp::min;
use std::path::PathBuf;

// From https://github.com/image-rs/image/issues/1958
pub fn get_jpeg_orientation(file_path: PathBuf) -> Result<u32, ()> {
    let file = std::fs::File::open(file_path).expect("problem opening the file");
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader
        .read_from_container(&mut bufreader)
        .expect("failed to read the exifreader");

    let orientation: u32 = match exif.get_field(Tag::Orientation, In::PRIMARY) {
        Some(orientation) => match orientation.value.get_uint(0) {
            Some(v @ 1..=8) => v,
            _ => 1,
        },
        None => 1,
    };

    Ok(orientation)
}

// From https://github.com/image-rs/image/issues/1958
fn rotate(mut img: DynamicImage, orientation: u8) -> DynamicImage {
    let rgba = img.color().has_alpha();
    img = match orientation {
        2 => DynamicImage::ImageRgba8(imageops::flip_horizontal(&img)),
        3 => DynamicImage::ImageRgba8(imageops::rotate180(&img)),
        4 => DynamicImage::ImageRgba8(imageops::flip_vertical(&img)),
        5 => DynamicImage::ImageRgba8(imageops::flip_horizontal(&imageops::rotate90(&img))),
        6 => DynamicImage::ImageRgba8(imageops::rotate90(&img)),
        7 => DynamicImage::ImageRgba8(imageops::flip_horizontal(&imageops::rotate270(&img))),
        8 => DynamicImage::ImageRgba8(imageops::rotate270(&img)),
        _ => img,
    };
    if !rgba {
        img = DynamicImage::ImageRgb8(img.into_rgb8());
    }
    img
}

fn load_jpg(path: &str) -> Result<DynamicImage, ()> {
    let orientation = get_jpeg_orientation(path.into())?;

    let img = image::open(path).unwrap();
    let img = rotate(img, orientation as u8);

    Ok(img)
}

#[derive(clap::Parser, Clone)]
struct MyArgs {
    /// Marign between images
    #[arg(short = 'm', default_value_t = 10)]
    margin: u32,

    /// Output width
    #[arg(short = 'w', default_value_t = 640)]
    width: u32,

    /// Stack vertically instead of horizontally
    #[arg(short = 'v')]
    vertical: bool,

    /// First image
    img1: String,

    /// Second image
    img2: String,
}

fn main() -> Result<(), ()> {
    let args = MyArgs::parse();

    let img1 = load_jpg(&args.img1)?;
    let img2 = load_jpg(&args.img2)?;

    let mut canvas;
    if args.vertical {
        let canvas_width = min(img1.width(), img2.width());
        let img1 = img1.resize(canvas_width, img1.height(), FilterType::Lanczos3);
        let img2 = img2.resize(canvas_width, img2.height(), FilterType::Lanczos3);
        canvas = DynamicImage::new(
            canvas_width,
            img1.height() + img2.height() + args.margin,
            img1.color(),
        );
        canvas.copy_from(&img1, 0, 0).unwrap();
        canvas
            .copy_from(&img2, 0, img1.height() + args.margin)
            .unwrap();
    } else {
        let canvas_height = min(img1.height(), img2.height());
        let img1 = img1.resize(img1.width(), canvas_height, FilterType::Lanczos3);
        let img2 = img2.resize(img2.width(), canvas_height, FilterType::Lanczos3);
        canvas = DynamicImage::new(
            img1.width() + img2.width() + args.margin,
            canvas_height,
            img1.color(),
        );
        canvas.copy_from(&img1, 0, 0).unwrap();
        canvas
            .copy_from(&img2, img1.width() + args.margin, 0)
            .unwrap();
    }

    // Only resize the width. Don't constrain by height.
    canvas = canvas.resize(args.width, u32::MAX, FilterType::Lanczos3);
    canvas.save("output.jpg").unwrap();

    Ok(())
}
