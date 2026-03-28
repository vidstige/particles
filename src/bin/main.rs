use std::{
    env,
    error::Error,
    io::{self, Write},
};

use particles::{
    render::{default_theme, render_background},
    resolution::Resolution,
};

fn default_resolution() -> Resolution {
    Resolution::new(512, 288)
}

fn resolution() -> Result<Resolution, Box<dyn Error>> {
    let resolution = match env::var("RESOLUTION") {
        Ok(value) => value.parse::<Resolution>()?,
        Err(env::VarError::NotPresent) => default_resolution(),
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut output = io::stdout().lock();
    let resolution = resolution()?;
    let theme = default_theme();
    let frame_count = 256;

    for _ in 0..frame_count {
        let pixmap = render_background(&resolution, &theme);
        output.write_all(pixmap.data())?;
        output.flush()?;
    }

    Ok(())
}
