use glam::Vec3;
use std::{fmt, str::FromStr};

pub struct DatVec3(pub Vec3);

impl fmt::Display for DatVec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.4}, {:.4}, {:.4})", self.0.x, self.0.y, self.0.z)
    }
}

impl FromStr for DatVec3 {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().strip_prefix('(').ok_or(())?.strip_suffix(')').ok_or(())?;
        let mut p = s.split(',');
        let x = p.next().ok_or(())?.trim().parse().map_err(|_| ())?;
        let y = p.next().ok_or(())?.trim().parse().map_err(|_| ())?;
        let z = p.next().ok_or(())?.trim().parse().map_err(|_| ())?;
        Ok(DatVec3(Vec3::new(x, y, z)))
    }
}
