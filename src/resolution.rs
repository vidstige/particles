use std::{fmt, str::FromStr};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ParseResolutionError;

impl fmt::Display for ParseResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expected RESOLUTION as WIDTHxHEIGHT with integers")
    }
}

impl std::error::Error for ParseResolutionError {}

impl Resolution {
    pub fn new(width: u32, height: u32) -> Resolution {
        Resolution { width, height }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn area(&self) -> usize {
        (self.width * self.height) as usize
    }
}

impl FromStr for Resolution {
    type Err = ParseResolutionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (width, height) = value.split_once('x').ok_or(ParseResolutionError)?;
        let width = width.parse().map_err(|_| ParseResolutionError)?;
        let height = height.parse().map_err(|_| ParseResolutionError)?;

        Ok(Self::new(width, height))
    }
}

#[cfg(test)]
mod tests {
    use super::Resolution;

    #[test]
    fn resolution_parses_width_by_height() {
        assert_eq!("512x288".parse(), Ok(Resolution::new(512, 288)));
    }

    #[test]
    fn resolution_rejects_invalid_values() {
        assert!("512".parse::<Resolution>().is_err());
        assert_eq!("0x288".parse(), Ok(Resolution::new(0, 288)));
        assert_eq!("512x0".parse(), Ok(Resolution::new(512, 0)));
        assert!("wide x tall".parse::<Resolution>().is_err());
    }
}
