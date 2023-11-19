pub mod config;
mod font;
mod util;

pub use config::Config;
pub use util::Error;

use chrono::{Duration, Utc};

pub fn run(config: crate::config::Config) -> Result<(), crate::util::Error> {
    use crate::util::*;
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
                        .and_then(|path| image.save(path).map_err(|e| Error::Image(e)))?;
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
                    .and_then(|path| i.save(path).map_err(|e| Error::Image(e)))?;
            }
            current_date = current_date + config.duration;
        }
    }
    Ok(())
}
