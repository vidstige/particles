use glam::Vec3;
use std::{fmt, fmt::Write as FmtWrite, fs, io, str::FromStr};

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

struct Entry {
    section: String,
    key: String,
    value: String,
}

pub struct Dat {
    entries: Vec<Entry>,
}

impl Dat {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn set(&mut self, section: &str, key: &str, value: &str) {
        self.entries.push(Entry {
            section: section.to_string(),
            key: key.to_string(),
            value: value.to_string(),
        });
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.entries.iter()
            .find(|e| e.section == section && e.key == key)
            .map(|e| e.value.as_str())
    }

    pub fn write(&self, path: &str) -> io::Result<()> {
        let mut out = String::new();
        let mut current_section = "";
        for entry in &self.entries {
            if entry.section != current_section {
                if !current_section.is_empty() {
                    out.push('\n');
                }
                if !entry.section.is_empty() {
                    let _ = writeln!(out, "[{}]", entry.section);
                }
                current_section = &entry.section;
            }
            let _ = writeln!(out, "{} = {}", entry.key, entry.value);
        }
        fs::write(path, out)
    }

    pub fn read(path: &str) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(Self::parse(&content))
    }

    fn parse(content: &str) -> Self {
        let mut dat = Self::new();
        let mut section = String::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                section = name.to_string();
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                dat.set(&section, key.trim(), value.trim());
            }
        }
        dat
    }
}
