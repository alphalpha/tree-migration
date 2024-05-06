use crate::{font, util};
use chrono::{DateTime, Duration, NaiveTime, TimeZone, Utc};
use confy;
use image::Rgb;
use imageproc::rect::Rect;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct RawConfig {
    pub images_path: String,
    pub roi: [i32; 4],
    pub font_path: String,
    pub font_size: f32,
    pub font_color: [u8; 3],
    pub location: String,
    pub camera: String,
    pub file_name: String,
    pub start_date: [u32; 3],
    pub end_date: [u32; 3],
    pub duration: i64,
    pub night_times: [u32; 2],
    pub night_color: [u8; 3],
    pub night_duration: i64,
}

impl ::std::default::Default for RawConfig {
    fn default() -> Self {
        Self {
            images_path: "".into(),
            roi: [0, 0, 0, 0],
            font_path: "".into(),
            font_size: 0.0,
            font_color: [0, 0, 0],
            location: "".into(),
            camera: "".into(),
            file_name: "".into(),
            start_date: [0, 0, 0],
            end_date: [0, 0, 0],
            duration: 0,
            night_times: [0, 0],
            night_color: [0, 0, 0],
            night_duration: 0,
        }
    }
}

#[derive(Clone)]
pub struct Config {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub roi: Rect,
    pub font: font::Font,
    pub location: String,
    pub camera: String,
    pub file_name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub duration: Duration,
    pub night_times: Option<(NaiveTime, NaiveTime)>,
    pub night_color: Rgb<u8>,
    pub night_duration: Duration,
    pub skip_night: bool,
}

impl Config {
    pub fn from(path: &Path) -> Result<Config, util::Error> {
        let raw_config: RawConfig = confy::load_path(path)?;
        let input_dir = Path::new(&raw_config.images_path).to_path_buf();
        if !input_dir.exists() {
            return Err(util::Error::Custom(format!(
                "Images Path {} does not exist",
                input_dir.display()
            )));
        }
        if let Ok(metadata) = input_dir.metadata() {
            if !metadata.is_dir() {
                return Err(util::Error::Custom(String::from(
                    "Input path is not a directory",
                )));
            };
        }

        // Always overwrite output dir
        let output_dir = input_dir.join(Path::new("Output"));
        if output_dir.exists() {
            fs::remove_dir_all(&output_dir)?;
        }
        fs::create_dir(&output_dir).map_err(|_| {
            util::Error::Custom(format!("{} already exists.", output_dir.display()))
        })?;

        let font = font::Font::new(
            Path::new(&raw_config.font_path),
            raw_config.font_size,
            Rgb(raw_config.font_color),
        )?;

        let mut night_times = None;
        if raw_config.night_times[0] != raw_config.night_times[1] {
            night_times = Some((
                NaiveTime::from_hms_opt(raw_config.night_times[0], 0, 0).ok_or(
                    util::Error::Custom("night_times [0] from_hms_opt fails".to_owned()),
                )?,
                NaiveTime::from_hms_opt(raw_config.night_times[1], 0, 0).ok_or(
                    util::Error::Custom("night_times [0] from_hms_opt fails".to_owned()),
                )?,
            ));
        }

        let night_duration = Duration::minutes(raw_config.night_duration);
        let mut skip_night = false;
        if night_duration == Duration::minutes(0) {
            skip_night = true;
        }

        Ok(Config {
            input_path: input_dir,
            output_path: output_dir,
            roi: Rect::at(raw_config.roi[0], raw_config.roi[1])
                .of_size(raw_config.roi[2] as u32, raw_config.roi[3] as u32),
            font: font,
            location: raw_config.location,
            camera: raw_config.camera,
            file_name: raw_config.file_name,
            start_date: Utc
                .with_ymd_and_hms(
                    raw_config.start_date[0] as i32,
                    raw_config.start_date[1],
                    raw_config.start_date[2],
                    0,
                    0,
                    0,
                )
                .unwrap(),
            end_date: Utc
                .with_ymd_and_hms(
                    raw_config.end_date[0] as i32,
                    raw_config.end_date[1],
                    raw_config.end_date[2],
                    23,
                    59,
                    59,
                )
                .unwrap(),
            duration: Duration::minutes(raw_config.duration),
            night_times,
            night_color: Rgb(raw_config.night_color),
            night_duration,
            skip_night,
        })
    }
}
