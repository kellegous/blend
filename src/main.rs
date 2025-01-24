use std::{error::Error, fmt, fs, io, str::FromStr};

use cairo::{Context, Format, ImageSurface};
use clap::Parser;
use jpeg_decoder::{Decoder, PixelFormat};
use jpeg_encoder::Encoder;

#[derive(Parser, Debug)]
struct Args {
    src: String,
    dst: String,
    #[clap(long, default_value = "0.5")]
    opacity: f64,

    #[clap(long, default_value_t = Color::white(), value_parser = Color::from_arg)]
    background: Color,

    #[clap(long, default_value_t = 60)]
    quality: u8,
}

#[derive(Debug, Clone)]
struct Color {
    r: u8,
    b: u8,
    g: u8,
}

impl Color {
    fn white() -> Color {
        Color {
            r: 255,
            g: 255,
            b: 255,
        }
    }

    fn from_arg(s: &str) -> Result<Color, String> {
        s.parse::<Color>().map_err(|e| e.to_string())
    }

    fn r(&self) -> f64 {
        self.r as f64 / 255.0
    }

    fn g(&self) -> f64 {
        self.g as f64 / 255.0
    }

    fn b(&self) -> f64 {
        self.b as f64 / 255.0
    }
}

impl FromStr for Color {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with('#') || s.len() != 7 {
            return Err(format!("Invalid color: {}", s).into());
        }
        let r = u8::from_str_radix(&s[1..3], 16).map_err(|_| format!("Invalid color: {}", s))?;
        let g = u8::from_str_radix(&s[3..5], 16).map_err(|_| format!("Invalid color: {}", s))?;
        let b = u8::from_str_radix(&s[5..7], 16).map_err(|_| format!("Invalid color: {}", s))?;
        Ok(Color { r, g, b })
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

fn decode_jpeg<R>(r: R) -> Result<ImageSurface, Box<dyn Error>>
where
    R: io::Read,
{
    let mut decoder = Decoder::new(io::BufReader::new(r));
    let rgb = decoder.decode()?;
    let metadata = decoder.info().ok_or("Failed to get metadata")?;
    if metadata.pixel_format != PixelFormat::RGB24 {
        return Err("Unsupported pixel format".into());
    }

    let width = metadata.width as usize;
    let height = metadata.height as usize;

    let mut rgba = Vec::with_capacity(width * height * 4);
    for chunk in rgb.chunks_exact(3) {
        rgba.push(chunk[2]);
        rgba.push(chunk[1]);
        rgba.push(chunk[0]);
        rgba.push(0);
    }

    let surface = ImageSurface::create_for_data(
        rgba.clone(),
        Format::Rgb24,
        width as i32,
        height as i32,
        Format::Rgb24.stride_for_width(width as u32)?,
    )?;

    Ok(surface)
}

fn encode_jpeg<W>(w: W, surface: ImageSurface, quality: u8) -> Result<(), Box<dyn Error>>
where
    W: io::Write,
{
    let encoder = Encoder::new(io::BufWriter::new(w), quality);
    let width = surface.width() as u16;
    let height = surface.height() as u16;
    let data = surface.take_data()?;
    encoder.encode(data.as_ref(), width, height, jpeg_encoder::ColorType::Bgra)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let photo = decode_jpeg(fs::File::open(&args.src)?)?;

    let surface = ImageSurface::create(Format::ARgb32, photo.width(), photo.height())?;
    {
        let ctx = Context::new(&surface)?;

        ctx.save()?;
        ctx.set_source_rgb(
            args.background.r(),
            args.background.g(),
            args.background.b(),
        );
        ctx.rectangle(0.0, 0.0, surface.width() as f64, surface.height() as f64);
        ctx.fill()?;
        ctx.restore()?;

        ctx.set_source_surface(photo, 0.0, 0.0)?;
        ctx.paint_with_alpha(args.opacity)?;
    }

    encode_jpeg(fs::File::create(&args.dst)?, surface, args.quality)?;

    Ok(())
}
