use confy::ConfyError;
use image::error::ImageError;
use std::{fmt, io, num};

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
