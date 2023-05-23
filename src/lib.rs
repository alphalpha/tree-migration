pub mod config;
mod font;
mod util;

pub use crate::config::Config;
pub use crate::util::Error;
use chrono::{DateTime, Duration, TimeZone, Utc};
use image::RgbImage;
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use imageproc::rect::Rect;
use rusttype::Point;
use std::fs;
use std::path::{Path, PathBuf};

fn image_paths(dir: &Path) -> Result<Vec<PathBuf>, util::Error> {
    let mut paths: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some())
        .collect();
    paths.sort();
    Ok(paths)
}

fn parse_date(name: &str, config: &Config) -> Result<(DateTime<Utc>, String), util::Error> {
    match Utc.datetime_from_str(name, config.file_name.as_str()) {
        Ok(date_time) => {
            let date = format!("{}", date_time.format("%d.%m.%Y, %H:%M:%S"));
            Ok((date_time, date))
        }
        _ => Err(util::Error::Custom(String::from(
            "File: \"".to_string() + name + "\" has wrong name format",
        ))),
    }
}

fn output_file_path(
    config: &Config,
    source_file: &Path,
    utc: &DateTime<Utc>,
) -> Result<PathBuf, util::Error> {
    let file_name =
        config.location.clone() + "-" + config.camera.as_str() + utc.to_string().as_str();
    let path =
        config
            .output_path
            .join(file_name)
            .with_extension(source_file.extension().ok_or_else(|| {
                util::Error::Custom(String::from("Could not obtain the file extension"))
            })?);
    println!("Save {:?}", path);
    Ok(path)
}

fn draw_citing(image: &mut RgbImage, config: &Config, position: &Point<i32>, text: &str) {
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

fn generate_image(
    config: &Config,
    in_image: &mut RgbImage,
    date: &String,
) -> Result<(), util::Error> {
    let mut position = Point {
        x: config.font.pos.0 as i32,
        y: config.font.pos.1 as i32,
    };
    let location_date = config.location.clone() + ", " + &config.camera + ", " + &date;
    draw_citing(in_image, &config, &position, &location_date.as_str());

    // let font_height = config.font.scale.y as i32;
    // position.y = config.font.pos.1 + font_height;
    // let title = "Location: 65°43'30.7\"N 27°23\'17.3\"E";
    // draw_citing(in_image, &config, &position, title);
    Ok(())
}

fn generate_night_image(
    config: &Config,
    dimensions: (u32, u32),
    current_date: &DateTime<Utc>,
) -> Result<RgbImage, util::Error> {
    let mut image = image::ImageBuffer::new(dimensions.0, dimensions.1);

    for (_x, _y, pixel) in image.enumerate_pixels_mut() {
        *pixel = config.night_color;
    }

    let mut position = Point {
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

fn date_from_file_name(
    file_path: &Path,
    config: &Config,
) -> Result<(DateTime<Utc>, String), util::Error> {
    file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| util::Error::Custom(String::from("Cannot obtain file name")))
        .and_then(|n| parse_date(n, config))
}

pub fn run(config: Config) -> Result<(), util::Error> {
    let input_paths = image_paths(&config.input_path)?;
    let mut current_date = config.start_date.clone();
    let mut file_iter = input_paths.iter();
    let mut file = file_iter.next().unwrap();
    let (_, mut date) = date_from_file_name(&file, &config)?;
    let mut night_end = None;

    while current_date < config.end_date {
        for p in input_paths.iter() {
            let (u, d) = date_from_file_name(&p, &config)?;
            if u > current_date {
                continue;
            }
            date = d;
            file = p;
        }
        let in_image = Some(image::open(&file)?.to_rgb8());

        if config.night_times.is_some()
            && (current_date.time() >= config.night_times.unwrap().0
                || current_date.time() < config.night_times.unwrap().1)
        {
            if config.skip_night {
                current_date = current_date + config.duration;
            } else {
                if night_end.is_none() {
                    let now = current_date
                        .date_naive()
                        .and_time(config.night_times.unwrap().1)
                        .and_local_timezone(Utc)
                        .unwrap();
                    let next = (current_date + Duration::days(1))
                        .date_naive()
                        .and_time(config.night_times.unwrap().1)
                        .and_local_timezone(Utc)
                        .unwrap();
                    if (next - current_date) < Duration::days(1) {
                        night_end = Some(next);
                    } else {
                        night_end = Some(now);
                    }
                }
                if let Some(i) = in_image {
                    let image = generate_night_image(&config, i.dimensions(), &current_date)?;
                    output_file_path(&config, &file, &current_date)
                        .and_then(|path| image.save(path).map_err(|e| util::Error::Image(e)))?;
                }

                if current_date + config.night_duration > night_end.unwrap() {
                    while current_date < night_end.unwrap() {
                        current_date = current_date + config.duration;
                    }
                    night_end = None;
                } else {
                    current_date = current_date + config.night_duration;
                }
            }
        } else {
            if let Some(mut i) = in_image {
                generate_image(&config, &mut i, &date)?;
                output_file_path(&config, &file, &current_date)
                    .and_then(|path| i.save(path).map_err(|e| util::Error::Image(e)))?;
            }
            current_date = current_date + config.duration;
        }
    }
    Ok(())
}
