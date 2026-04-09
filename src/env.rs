use std::{env, error::Error, io};

use crate::resolution::Resolution;

pub const DEFAULT_FPS: f32 = 30.0;
pub const DEFAULT_RESOLUTION: Resolution = Resolution::new(512, 288);

pub fn fps() -> Result<f32, Box<dyn Error>> {
    match env::var("FPS") {
        Ok(value) => Ok(value.parse::<f32>()?),
        Err(env::VarError::NotPresent) => Ok(DEFAULT_FPS),
        Err(error) => Err(error.into()),
    }
}

pub fn resolution() -> Result<Resolution, Box<dyn Error>> {
    let resolution = match env::var("RESOLUTION") {
        Ok(value) => value.parse::<Resolution>()?,
        Err(env::VarError::NotPresent) => DEFAULT_RESOLUTION,
        Err(error) => return Err(error.into()),
    };

    if resolution.area() == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "RESOLUTION must have non-zero area",
        )
        .into());
    }

    Ok(resolution)
}
