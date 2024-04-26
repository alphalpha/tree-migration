use confy::ConfyError;
use image::error::ImageError;
use std::{fmt, io, num};

use crate::font;
use chrono::{DateTime, TimeZone, Utc};
use image::{GenericImage, ImageBuffer, Rgb, RgbImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use imageproc::rect::Rect;
use rusttype::Point;
use std::fs;
use std::path::{Path, PathBuf};

fn crop_image(image: &mut RgbImage, rect: &Rect) -> Result<RgbImage, Error> {
    Ok(image
        .sub_image(
            rect.left() as u32,
            rect.top() as u32,
            rect.width(),
            rect.height(),
        )
        .to_image())
}

fn mean_color(image: &RgbImage) -> Result<Rgb<u8>, Error> {
    let num_pixels = image.width() * image.height();
    let color: Vec<u8> = image
        .pixels()
        .fold(vec![0u32, 0u32, 0u32], |mut acc, pixel| {
            for i in 0..acc.len() {
                acc[i] += pixel[i] as u32;
            }
            acc
        })
        .iter()
        .map(|c| (c / num_pixels) as u8)
        .collect();

    Ok(Rgb([color[0], color[1], color[2]]))
}

pub fn image_paths(dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut paths: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some())
        .collect();
    paths.sort();
    Ok(paths)
}

pub fn parse_date(
    name: &str,
    config: &crate::config::Config,
) -> Result<(DateTime<Utc>, String), Error> {
    match Utc.datetime_from_str(name, config.file_name.as_str()) {
        Ok(date_time) => {
            let date = format!("{}", date_time.format("%d.%m.%Y, %H:%M:%S"));
            Ok((date_time, date))
        }
        _ => Err(Error::Custom(String::from(
            "File: \"".to_string() + name + "\" has wrong name format",
        ))),
    }
}

pub fn output_file_path(
    config: &crate::config::Config,
    source_file: &Path,
    utc: &DateTime<Utc>,
) -> Result<PathBuf, Error> {
    let file_name =
        config.location.clone() + "-" + config.camera.as_str() + utc.to_string().as_str();
    let path = config.output_path.join(file_name).with_extension(
        source_file
            .extension()
            .ok_or_else(|| Error::Custom(String::from("Could not obtain the file extension")))?,
    );
    Ok(path)
}

pub fn draw_citing(
    image: &mut RgbImage,
    config: &crate::config::Config,
    position: &Point<i32>,
    text: &str,
) {
    if let Some(width) = font::text_width(config.font.scale, &config.font.font, text) {
        let height = config.font.scale.y as u32;
        draw_filled_rect_mut(
            image,
            Rect::at(position.x as i32, position.y as i32).of_size(width, height),
            config.font.background_color,
        );
        draw_text_mut(
            image,
            config.font.color,
            position.x,
            position.y,
            config.font.scale,
            &config.font.font,
            text,
        );
    }
}

pub fn generate_image(
    config: &crate::config::Config,
    in_image: &mut RgbImage,
    date: &String,
    forest_green: bool,
) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, Error> {
    let position = Point {
        x: config.font.pos.0 as i32,
        y: config.font.pos.1 as i32,
    };
    let location_date = config.location.clone() + ", " + &config.camera + ", " + &date;
    draw_citing(in_image, &config, &position, &location_date.as_str());

    if forest_green {
        let color = crop_image(in_image, &config.roi).and_then(|i| mean_color(&i))?;
        let dimensions = in_image.dimensions();
        let mut image = image::ImageBuffer::new(2 * dimensions.0, dimensions.1);

        for (x, y, pixel) in image.enumerate_pixels_mut() {
            if x < dimensions.0 {
                *pixel = color;
            } else {
                *pixel = *in_image.get_pixel(x - dimensions.0, y);
            }
        }

        let mut position = Point {
            x: config.font.pos.0,
            y: config.font.pos.1,
        };
        let location_date = config.location.clone() + ", " + &date;
        draw_citing(&mut image, &config, &position, &location_date.as_str());

        let font_height = config.font.scale.y;
        position.y = config.font.pos.1 + font_height as i32;
        let title = "Average colour of vegetation";
        draw_citing(&mut image, &config, &position, title);

        position.y = config.font.pos.1 + 2 * font_height as i32;
        let color_string = format!("{:?}", color);
        draw_citing(&mut image, &config, &position, &color_string.as_str());
        Ok(image)
    } else {
        Ok(in_image.clone())
    }
}

pub fn generate_night_image(
    config: &crate::config::Config,
    dimensions: (u32, u32),
    current_date: &DateTime<Utc>,
    forest_green: bool,
) -> Result<RgbImage, Error> {
    let mut image = if forest_green {
        image::ImageBuffer::new(2 * dimensions.0, dimensions.1)
    } else {
        image::ImageBuffer::new(dimensions.0, dimensions.1)
    };

    for (_x, _y, pixel) in image.enumerate_pixels_mut() {
        *pixel = config.night_color;
    }

    let position = Point {
        x: config.font.pos.0,
        y: config.font.pos.1,
    };
    let date = format!("{}", current_date.format("%d.%m.%Y, %T"));
    let location_date = config.location.clone() + ", " + &config.camera + ", " + &date;
    draw_citing(&mut image, &config, &position, &location_date.as_str());

    // let font_height = config.font.scale.y as i32;
    // position.y = config.font.pos.1 + font_height;
    // let title = "Location: 65°43'30.7\"N 27°23\'17.3\"E";
    // draw_citing(&mut image, &config, &position, title);

    Ok(image)
}

pub fn date_from_file_name(
    file_path: &Path,
    config: &crate::config::Config,
) -> Result<(DateTime<Utc>, String), Error> {
    file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| Error::Custom(String::from("Cannot obtain file name")))
        .and_then(|n| parse_date(n, config))
}

#[derive(Debug)]
pub enum Error {
    Image(ImageError),
    Io(io::Error),
    ParseFloat(num::ParseFloatError),
    ParseInt(num::ParseIntError),
    Custom(String),
    Config(ConfyError),
    Else,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Image(ref err) => write!(f, "Image Error: {}", err),
            Error::Io(ref err) => write!(f, "IO Error: {}", err),
            Error::ParseFloat(ref err) => write!(f, "Parse Error: {}", err),
            Error::ParseInt(ref err) => write!(f, "Parse Error: {}", err),
            Error::Custom(ref err) => write!(f, "Error: {}", err),
            Error::Config(ref err) => write!(f, "Error: {}", err),
            Error::Else => write!(f, "Some Error"),
        }
    }
}

impl From<ImageError> for Error {
    fn from(err: ImageError) -> Error {
        Error::Image(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<num::ParseFloatError> for Error {
    fn from(err: num::ParseFloatError) -> Error {
        Error::ParseFloat(err)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Error {
        Error::ParseInt(err)
    }
}

impl From<ConfyError> for Error {
    fn from(err: ConfyError) -> Error {
        Error::Config(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::Custom(err)
    }
}
