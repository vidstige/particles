use crate::{data::Dat, glitter::Glitter, vec3_fmt::DatVec3};

const SECTION: &str = "glitter";

pub fn save_glitter(dat: &mut Dat, glitter: Glitter) {
    dat.set(SECTION, "falloff_power", &format!("{:.4}", glitter.falloff_power));
    dat.set(SECTION, "axis0_speed",   &format!("{:.4}", glitter.axis0_speed));
    dat.set(SECTION, "axis0",         &DatVec3(glitter.axis0).to_string());
    dat.set(SECTION, "axis1",         &DatVec3(glitter.axis1).to_string());
    dat.set(SECTION, "axis1_speed",   &format!("{:.4}", glitter.axis1_speed));
}

pub fn load_glitter(dat: &Dat, default: Glitter) -> Glitter {
    let mut g = default;
    if let Some(v) = dat.get(SECTION, "falloff_power").and_then(|s| s.parse().ok()) { g.falloff_power = v; }
    if let Some(v) = dat.get(SECTION, "axis0_speed").and_then(|s| s.parse().ok())   { g.axis0_speed = v; }
    if let Some(v) = dat.get(SECTION, "axis0").and_then(|s| s.parse::<DatVec3>().ok()) { g.axis0 = v.0; }
    if let Some(v) = dat.get(SECTION, "axis1").and_then(|s| s.parse::<DatVec3>().ok()) { g.axis1 = v.0; }
    if let Some(v) = dat.get(SECTION, "axis1_speed").and_then(|s| s.parse().ok())   { g.axis1_speed = v; }
    g
}
