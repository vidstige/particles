use std::{collections::HashMap, fs, io};

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = !0;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB88320 } else { crc >> 1 };
        }
    }
    !crc
}

fn make_npy(shape: &[usize], data: &[f32]) -> Vec<u8> {
    let shape_str = match shape {
        [n] => format!("({n},)"),
        _   => {
            let parts: Vec<String> = shape.iter().map(|n| n.to_string()).collect();
            format!("({})", parts.join(", "))
        }
    };
    let dict = format!("{{'descr': '<f4', 'fortran_order': False, 'shape': {shape_str}, }}");

    // Total (10 prefix bytes + header_len) must be a multiple of 64
    let min_len = dict.len() + 1; // dict + '\n'
    let header_len = ((10 + min_len + 63) / 64) * 64 - 10;
    let padding = header_len - min_len;

    let mut out = Vec::with_capacity(10 + header_len + data.len() * 4);
    out.extend_from_slice(b"\x93NUMPY\x01\x00");
    out.extend_from_slice(&(header_len as u16).to_le_bytes());
    out.extend_from_slice(dict.as_bytes());
    out.extend(std::iter::repeat(b' ').take(padding));
    out.push(b'\n');
    for &v in data {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

struct ZipEntry {
    name: String,
    data: Vec<u8>,
    crc: u32,
    offset: u32,
}

pub struct Npz {
    entries: Vec<ZipEntry>,
}

impl Npz {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add(&mut self, name: &str, shape: &[usize], data: &[f32]) {
        let npy = make_npy(shape, data);
        let crc = crc32(&npy);
        self.entries.push(ZipEntry {
            name: format!("{name}.npy"),
            data: npy,
            crc,
            offset: 0,
        });
    }

    pub fn write(&mut self, path: &str) -> io::Result<()> {
        let mut out = Vec::new();

        for entry in &mut self.entries {
            entry.offset = out.len() as u32;
            write_local_header(&mut out, &entry.name, entry.data.len() as u32, entry.crc);
            out.extend_from_slice(&entry.data);
        }

        let central_dir_offset = out.len() as u32;
        for entry in &self.entries {
            write_central_header(&mut out, &entry.name, entry.data.len() as u32, entry.crc, entry.offset);
        }
        let central_dir_size = out.len() as u32 - central_dir_offset;
        write_eocd(&mut out, self.entries.len() as u16, central_dir_size, central_dir_offset);

        fs::write(path, out)
    }
}

fn u16le(out: &mut Vec<u8>, v: u16) { out.extend_from_slice(&v.to_le_bytes()); }
fn u32le(out: &mut Vec<u8>, v: u32) { out.extend_from_slice(&v.to_le_bytes()); }

fn write_local_header(out: &mut Vec<u8>, name: &str, size: u32, crc: u32) {
    u32le(out, 0x04034b50);
    u16le(out, 20); u16le(out, 0); u16le(out, 0); // version, flags, compression
    u16le(out, 0); u16le(out, 0);                  // mod time, mod date
    u32le(out, crc); u32le(out, size); u32le(out, size);
    u16le(out, name.len() as u16); u16le(out, 0);  // filename len, extra len
    out.extend_from_slice(name.as_bytes());
}

fn write_central_header(out: &mut Vec<u8>, name: &str, size: u32, crc: u32, offset: u32) {
    u32le(out, 0x02014b50);
    u16le(out, 20); u16le(out, 20); u16le(out, 0); u16le(out, 0); // versions, flags, compression
    u16le(out, 0); u16le(out, 0);                                  // mod time, mod date
    u32le(out, crc); u32le(out, size); u32le(out, size);
    u16le(out, name.len() as u16); u16le(out, 0); u16le(out, 0);  // filename, extra, comment lens
    u16le(out, 0); u16le(out, 0); u32le(out, 0);                  // disk start, internal/external attrs
    u32le(out, offset);
    out.extend_from_slice(name.as_bytes());
}

fn write_eocd(out: &mut Vec<u8>, count: u16, dir_size: u32, dir_offset: u32) {
    u32le(out, 0x06054b50);
    u16le(out, 0); u16le(out, 0);    // disk number, central dir disk
    u16le(out, count); u16le(out, count);
    u32le(out, dir_size); u32le(out, dir_offset);
    u16le(out, 0); // comment length
}

// --- reading ---

pub struct NpyArray {
    pub shape: Vec<usize>,
    pub data: Vec<f32>,
}

fn r16(b: &[u8], o: usize) -> u16 { u16::from_le_bytes([b[o], b[o+1]]) }
fn r32(b: &[u8], o: usize) -> u32 { u32::from_le_bytes([b[o], b[o+1], b[o+2], b[o+3]]) }

fn parse_shape(header: &str) -> Option<Vec<usize>> {
    let start = header.find("'shape': (")? + "'shape': (".len();
    let end   = start + header[start..].find(')')?;
    header[start..end].split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect::<Vec<usize>>()
        .into_iter()
        .collect::<Vec<_>>()
        .into()
}

fn parse_npy(b: &[u8]) -> Option<NpyArray> {
    if b.len() < 10 || &b[..6] != b"\x93NUMPY" { return None; }
    let header_len = r16(b, 8) as usize;
    let header = std::str::from_utf8(&b[10..10 + header_len]).ok()?;
    let shape = parse_shape(header)?;
    let floats = b[10 + header_len..].chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    Some(NpyArray { shape, data: floats })
}

pub fn read_npz(path: &str) -> io::Result<HashMap<String, NpyArray>> {
    let b = fs::read(path)?;
    let mut result = HashMap::new();

    if b.len() < 22 || r32(&b, b.len() - 22) != 0x06054b50 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "not a ZIP file"));
    }
    let cd_offset = r32(&b, b.len() - 22 + 16) as usize;
    let count     = r16(&b, b.len() - 22 + 10) as usize;

    let mut pos = cd_offset;
    for _ in 0..count {
        if r32(&b, pos) != 0x02014b50 { break; }
        let fname_len   = r16(&b, pos + 28) as usize;
        let extra_len   = r16(&b, pos + 30) as usize;
        let comment_len = r16(&b, pos + 32) as usize;
        let local_off   = r32(&b, pos + 42) as usize;
        let name = String::from_utf8_lossy(&b[pos + 46..pos + 46 + fname_len]).into_owned();
        pos += 46 + fname_len + extra_len + comment_len;

        if r32(&b, local_off) != 0x04034b50 { continue; }
        let lname_len = r16(&b, local_off + 26) as usize;
        let lextra_len = r16(&b, local_off + 28) as usize;
        let data_off  = local_off + 30 + lname_len + lextra_len;
        let data_size = r32(&b, local_off + 18) as usize;

        let key = name.strip_suffix(".npy").unwrap_or(&name).to_string();
        if let Some(array) = parse_npy(&b[data_off..data_off + data_size]) {
            result.insert(key, array);
        }
    }
    Ok(result)
}
