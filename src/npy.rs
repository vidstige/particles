use std::{fs, io};

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
