//! Byte-exact emulation of the ZIP files CPython's `zipfile` writes for
//! `ZipInfo(name, (Y, M, D, 0, 0, 0))` entries with `ZIP_DEFLATED`, `compresslevel=9` and
//! `external_attr = 0o644 << 16`, plus a reader that checks every entry like `testzip()`.
//!
//! Layout per entry: a 30-byte local header with the final CRC and sizes (no data
//! descriptor, no extra field), the raw deflate stream, then one central-directory record
//! per entry in write order and the end record. Duplicate names are written as separate
//! entries, as CPython does. Deflate is classic C zlib (level 9, window 15, memLevel 8,
//! default strategy): other implementations produce different bytes.

use flate2::{Compress, Compression, Crc, Decompress, FlushCompress, FlushDecompress, Status};
use gai_core::parallel::parallel_map;

/// Largest size or offset CPython writes without switching to ZIP64.
const ZIP64_LIMIT: u64 = (1 << 31) - 1;
/// Largest entry count CPython writes without a ZIP64 end record.
const FILECOUNT_LIMIT: usize = 0xFFFF;
const LOCAL_HEADER: &[u8; 4] = b"PK\x03\x04";
const CENTRAL_HEADER: &[u8; 4] = b"PK\x01\x02";
const END_RECORD: &[u8; 4] = b"PK\x05\x06";
/// `0o644 << 16`: Unix mode bits without `S_IFREG`, as the Python packagers set them.
const EXTERNAL_ATTR: u32 = 0o644 << 16;
/// Unicode file-name flag (bit 11), set only for names that are not ASCII.
const UTF8_FLAG: u16 = 0x0800;

/// Errors from writing or reading a ZIP archive.
#[derive(Debug, thiserror::Error)]
pub enum ZipError {
    /// CPython would have written ZIP64 structures here; this writer refuses instead.
    #[error("{0} exceeds the non-ZIP64 limit")]
    TooLarge(String),
    /// The archive is malformed, uses an unsupported feature, or fails its CRC check.
    #[error("invalid archive: {0}")]
    Invalid(String),
    /// The archive holds more members or bytes than the reader's [`ZipLimits`] allow.
    #[error("archive exceeds the reader's limits: {0}")]
    Limit(String),
}

/// Bounds on what [`read_zip_with`] reads, so a crafted archive cannot exhaust memory:
/// sizes are the declared uncompressed sizes, checked before anything is inflated, and no
/// member may inflate past its declared size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZipLimits {
    /// Most members.
    pub entries: usize,
    /// Largest member, uncompressed.
    pub entry_bytes: usize,
    /// Largest sum of the members, uncompressed.
    pub total_bytes: usize,
}

impl ZipLimits {
    /// What [`read_zip`] allows: as many members as the format holds, 512 MiB each and
    /// 2 GiB in total (`icon-library.zip` has about 6,300 members and 265 MB).
    pub const ARCHIVES: ZipLimits = ZipLimits {
        entries: FILECOUNT_LIMIT,
        entry_bytes: 512 << 20,
        total_bytes: 2 << 30,
    };
    /// A `.lottie` file from outside the library (dotlottie-rs's limits): 4,096 members,
    /// 64 MiB each and 256 MiB in total.
    pub const LOTTIE: ZipLimits = ZipLimits {
        entries: 4096,
        entry_bytes: 64 << 20,
        total_bytes: 256 << 20,
    };
}

/// The fixed modification date of every entry (the time is always 00:00:00).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZipDate {
    year: u16,
    month: u8,
    day: u8,
}

impl ZipDate {
    /// `.lottie` files and set zips of the legacy sets 01–05.
    pub const LEGACY_SETS: ZipDate = ZipDate::new(2026, 9, 6);
    /// `icon-library.zip`.
    pub const ICON_LIBRARY: ZipDate = ZipDate::new(2026, 9, 7);
    /// `.lottie` files and set zips of every other set.
    pub const SETS: ZipDate = ZipDate::new(2026, 9, 8);
    /// Extras appended to set 16's zip.
    pub const VOICE_EXTRAS: ZipDate = ZipDate::new(2026, 9, 14);
    /// Extras appended to the zips of sets 17–34.
    pub const REVIEW_EXTRAS: ZipDate = ZipDate::new(2026, 9, 15);

    /// A date representable in DOS format (1980–2107).
    const fn new(year: u16, month: u8, day: u8) -> Self {
        assert!(year >= 1980 && year <= 2107 && month >= 1 && month <= 12 && day >= 1 && day <= 31);
        ZipDate { year, month, day }
    }

    fn dos(self) -> u16 {
        ((self.year - 1980) << 9) | (u16::from(self.month) << 5) | u16::from(self.day)
    }
}

/// One compressed archive member. Compression happens once, so the same entry can be
/// written into several archives (every `.lottie` of an icon holds the same actions).
#[derive(Debug, Clone)]
pub struct ZipEntry {
    name: String,
    date: ZipDate,
    crc: u32,
    size: u64,
    compressed: Vec<u8>,
}

impl ZipEntry {
    /// Deflates `data` as CPython's `writestr(..., compresslevel=9)` does.
    pub fn deflate(name: impl Into<String>, data: &[u8], date: ZipDate) -> Self {
        let mut crc = Crc::new();
        crc.update(data);
        ZipEntry {
            name: name.into(),
            date,
            crc: crc.sum(),
            size: data.len() as u64,
            compressed: deflate_raw9(data),
        }
    }
}

/// Deflates many `(name, data)` pairs on all cores, keeping their order.
pub fn deflate_all(items: Vec<(String, Vec<u8>)>, date: ZipDate) -> Vec<ZipEntry> {
    parallel_map(items, |(name, data)| ZipEntry::deflate(name, &data, date))
}

fn deflate_raw9(data: &[u8]) -> Vec<u8> {
    // Raw deflate (window bits -15), memLevel 8, default strategy: zlib.compressobj(9, DEFLATED, -15).
    let mut stream = Compress::new(Compression::new(9), false);
    let mut out = Vec::with_capacity(data.len() / 3 + 64);
    loop {
        let consumed = stream.total_in() as usize;
        out.reserve(64 * 1024);
        let status = stream
            .compress_vec(&data[consumed..], &mut out, FlushCompress::Finish)
            .expect("zlib deflate with valid parameters cannot fail");
        if status == Status::StreamEnd {
            return out;
        }
    }
}

/// Serializes `entries` in order into a complete archive.
pub fn write_zip(entries: &[ZipEntry]) -> Result<Vec<u8>, ZipError> {
    if entries.len() > FILECOUNT_LIMIT {
        return Err(ZipError::TooLarge(format!("entry count {}", entries.len())));
    }
    let mut out = Vec::with_capacity(
        entries
            .iter()
            .map(|e| e.compressed.len() + 2 * e.name.len() + 76)
            .sum::<usize>()
            + 22,
    );
    let mut central = Vec::new();
    for entry in entries {
        // CPython switches to ZIP64 when size * 1.05 exceeds the limit, before compressing.
        if entry.size as f64 * 1.05 > ZIP64_LIMIT as f64
            || entry.compressed.len() as u64 > ZIP64_LIMIT
        {
            return Err(ZipError::TooLarge(format!("entry {}", entry.name)));
        }
        let name_len = u16::try_from(entry.name.len())
            .map_err(|_| ZipError::TooLarge(format!("name {}", entry.name)))?;
        let offset = out.len() as u64;
        if offset > ZIP64_LIMIT {
            return Err(ZipError::TooLarge(format!("offset of {}", entry.name)));
        }
        let flags = if entry.name.is_ascii() { 0 } else { UTF8_FLAG };
        let common = |buf: &mut Vec<u8>| {
            for value in [flags, 8, 0, entry.date.dos()] {
                buf.extend_from_slice(&value.to_le_bytes()); // flags, method, time, date
            }
            for value in [entry.crc, entry.compressed.len() as u32, entry.size as u32] {
                buf.extend_from_slice(&value.to_le_bytes());
            }
            buf.extend_from_slice(&name_len.to_le_bytes());
            buf.extend_from_slice(&0u16.to_le_bytes()); // extra length
        };

        out.extend_from_slice(LOCAL_HEADER);
        out.extend_from_slice(&[20, 0]); // extract version, reserved
        common(&mut out);
        out.extend_from_slice(entry.name.as_bytes());
        out.extend_from_slice(&entry.compressed);

        central.extend_from_slice(CENTRAL_HEADER);
        central.extend_from_slice(&[20, 3, 20, 0]); // create version, Unix, extract version, reserved
        common(&mut central);
        for value in [0u16, 0, 0] {
            central.extend_from_slice(&value.to_le_bytes()); // comment length, disk, internal attributes
        }
        central.extend_from_slice(&EXTERNAL_ATTR.to_le_bytes());
        central.extend_from_slice(&(offset as u32).to_le_bytes());
        central.extend_from_slice(entry.name.as_bytes());
    }
    let (cd_offset, cd_size) = (out.len() as u64, central.len() as u64);
    if cd_offset > ZIP64_LIMIT || cd_size > ZIP64_LIMIT {
        return Err(ZipError::TooLarge("central directory".into()));
    }
    out.extend_from_slice(&central);
    out.extend_from_slice(END_RECORD);
    let count = entries.len() as u16;
    for value in [0u16, 0, count, count] {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out.extend_from_slice(&(cd_size as u32).to_le_bytes());
    out.extend_from_slice(&(cd_offset as u32).to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // comment length
    Ok(out)
}

/// One member read back from an archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZipFile {
    /// The entry name.
    pub name: String,
    /// The uncompressed bytes.
    pub data: Vec<u8>,
}

/// All members of an archive in central-directory order, each inflated and CRC-checked
/// (the equivalent of `testzip()` returning `None`), within [`ZipLimits::ARCHIVES`].
/// Handles stored and deflated entries of non-ZIP64, unencrypted archives.
pub fn read_zip(bytes: &[u8]) -> Result<Vec<ZipFile>, ZipError> {
    read_zip_with(bytes, &ZipLimits::ARCHIVES)
}

/// [`read_zip`] within `limits`; use [`ZipLimits::LOTTIE`] for files a project supplies.
pub fn read_zip_with(bytes: &[u8], limits: &ZipLimits) -> Result<Vec<ZipFile>, ZipError> {
    let invalid = |what: &str| ZipError::Invalid(what.to_string());
    let end =
        find_end_record(bytes).ok_or_else(|| invalid("no end of central directory record"))?;
    let count = usize::from(u16_at(bytes, end + 10));
    if count > limits.entries {
        return Err(ZipError::Limit(format!(
            "{count} members, more than {}",
            limits.entries
        )));
    }
    let cd_offset = u32_at(bytes, end + 16) as usize;
    let mut files = Vec::with_capacity(count);
    let mut total = 0usize;
    let mut at = cd_offset;
    for _ in 0..count {
        if bytes.get(at..at + 4) != Some(CENTRAL_HEADER.as_slice()) || at + 46 > bytes.len() {
            return Err(invalid("bad central directory record"));
        }
        let flags = u16_at(bytes, at + 8);
        let method = u16_at(bytes, at + 10);
        let crc = u32_at(bytes, at + 16);
        let compressed_size = u32_at(bytes, at + 20) as usize;
        let size = u32_at(bytes, at + 24) as usize;
        let name_len = usize::from(u16_at(bytes, at + 28));
        let extra_len = usize::from(u16_at(bytes, at + 30));
        let comment_len = usize::from(u16_at(bytes, at + 32));
        let local = u32_at(bytes, at + 42) as usize;
        let name_bytes = bytes
            .get(at + 46..at + 46 + name_len)
            .ok_or_else(|| invalid("truncated name"))?;
        let name = if flags & UTF8_FLAG != 0 {
            String::from_utf8(name_bytes.to_vec()).map_err(|_| invalid("name is not UTF-8"))?
        } else {
            name_bytes.iter().map(|&b| char::from(b)).collect() // cp437 for ASCII is identity
        };
        if flags & 1 != 0 {
            return Err(ZipError::Invalid(format!(
                "{name}: encrypted entries are not supported"
            )));
        }
        if bytes.get(local..local + 4) != Some(LOCAL_HEADER.as_slice()) || local + 30 > bytes.len()
        {
            return Err(ZipError::Invalid(format!("{name}: bad local header")));
        }
        // Checked before any allocation; many members may share one deflate stream.
        total = total.saturating_add(size);
        if size > limits.entry_bytes || total > limits.total_bytes {
            return Err(ZipError::Limit(format!(
                "{name}: {size} bytes, past {} per member or {} in total",
                limits.entry_bytes, limits.total_bytes
            )));
        }
        let data_start = local
            + 30
            + usize::from(u16_at(bytes, local + 26))
            + usize::from(u16_at(bytes, local + 28));
        let raw = bytes
            .get(data_start..data_start + compressed_size)
            .ok_or_else(|| ZipError::Invalid(format!("{name}: truncated data")))?;
        let data = match method {
            0 if raw.len() == size => raw.to_vec(),
            0 => return Err(ZipError::Invalid(format!("{name}: CRC or size mismatch"))),
            8 => inflate_raw(raw, size).map_err(|error| match error {
                Inflate::Corrupt => ZipError::Invalid(format!("{name}: corrupt deflate stream")),
                Inflate::PastSize => {
                    ZipError::Limit(format!("{name} inflates past its declared {size} bytes"))
                }
            })?,
            other => {
                return Err(ZipError::Invalid(format!(
                    "{name}: compression method {other}"
                )));
            }
        };
        let mut check = Crc::new();
        check.update(&data);
        if data.len() != size || check.sum() != crc {
            return Err(ZipError::Invalid(format!("{name}: CRC or size mismatch")));
        }
        files.push(ZipFile { name, data });
        at += 46 + name_len + extra_len + comment_len;
    }
    Ok(files)
}

/// The data of the last member called `name` (CPython's `read(name)` for duplicates).
pub fn find_file<'a>(files: &'a [ZipFile], name: &str) -> Option<&'a [u8]> {
    files
        .iter()
        .rev()
        .find(|f| f.name == name)
        .map(|f| f.data.as_slice())
}

/// Why a member did not inflate.
enum Inflate {
    /// Not a complete deflate stream.
    Corrupt,
    /// It produces more than its declared size.
    PastSize,
}

/// Output grows by at most this much at a time, so a false declared size costs nothing.
const INFLATE_STEP: usize = 1 << 20;

/// Inflates a raw deflate stream that should produce `size` bytes, never holding more than
/// one byte past `size` (plus allocator slack).
fn inflate_raw(raw: &[u8], size: usize) -> Result<Vec<u8>, Inflate> {
    let mut stream = Decompress::new(false);
    // Room for one byte past `size` shows a stream that inflates further.
    let limit = size.saturating_add(1);
    let mut out = Vec::with_capacity(limit.min(INFLATE_STEP));
    loop {
        let consumed = stream.total_in() as usize;
        if out.len() == out.capacity() {
            if out.len() >= limit {
                return Err(Inflate::PastSize);
            }
            out.reserve_exact((limit - out.len()).min(INFLATE_STEP));
        }
        let status = stream
            .decompress_vec(&raw[consumed..], &mut out, FlushDecompress::Finish)
            .map_err(|_| Inflate::Corrupt)?;
        match status {
            Status::StreamEnd if out.len() > size => return Err(Inflate::PastSize),
            Status::StreamEnd => return Ok(out),
            // No progress with output space left means the stream is truncated.
            _ if stream.total_in() as usize == consumed && out.len() < out.capacity() => {
                return Err(Inflate::Corrupt);
            }
            _ => {}
        }
    }
}

fn find_end_record(bytes: &[u8]) -> Option<usize> {
    let last = bytes.len().checked_sub(22)?;
    let first = last.saturating_sub(0xFFFF);
    (first..=last)
        .rev()
        .find(|&i| &bytes[i..i + 4] == END_RECORD)
}

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    bytes
        .get(at..at + 2)
        .map_or(0, |b| u16::from_le_bytes([b[0], b[1]]))
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    bytes
        .get(at..at + 4)
        .map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gai_core::sha256_hex;

    fn archive(entries: &[(&str, &[u8])], date: ZipDate) -> Vec<u8> {
        let entries: Vec<ZipEntry> = entries
            .iter()
            .map(|(n, d)| ZipEntry::deflate(*n, d, date))
            .collect();
        write_zip(&entries).unwrap()
    }

    #[test]
    fn empty_entry_deflates_to_final_empty_block() {
        assert_eq!(deflate_raw9(b""), [0x03, 0x00]);
    }

    #[test]
    fn matches_cpython_zipfile() {
        // python3: ZipInfo(n, (2026, 9, 8, 0, 0, 0)), ZIP_DEFLATED, external_attr 0o644 << 16,
        // writestr(..., compresslevel=9) for the same three entries.
        let json = "{\"x\": 1}".repeat(50);
        let bytes = archive(
            &[
                ("a.json", json.as_bytes()),
                ("b/c.txt", b""),
                ("a.json", b"dup"),
            ],
            ZipDate::SETS,
        );
        assert_eq!(bytes.len(), 310);
        assert_eq!(
            sha256_hex(&bytes),
            "1877dc641538811200925e9db693c4350037da556498a5f08538f1ae09c34871"
        );
    }

    #[test]
    fn deflate_is_classic_zlib() {
        // zlib.compressobj(9, zlib.DEFLATED, -15) of this text; miniz_oxide and zlib-rs
        // (other flate2 backends) emit different streams for it.
        let mut x: u64 = 12345;
        let parts: Vec<String> = (0..300)
            .map(|i| {
                x = (x * 1_103_515_245 + 12345) % (1 << 31);
                format!(
                    "{{\"t\":{},\"s\":[{}.{:03},{}.{}]}}",
                    i % 60,
                    x % 33,
                    (x >> 8) % 1000,
                    (x >> 4) % 17,
                    (x >> 12) % 10
                )
            })
            .collect();
        let text = format!("{{\"k\":[{}]}}", parts.join(","));
        assert_eq!(text.len(), 7788);
        assert_eq!(
            sha256_hex(deflate_raw9(text.as_bytes())),
            "3a15bdd0f4b824e571f10d65727a75c1b1f5e18a9b63b1caa70df9d29d7ce024"
        );
    }

    #[test]
    fn round_trips_and_keeps_duplicates() {
        let bytes = archive(
            &[("x", b"one"), ("é", b"two"), ("x", b"three")],
            ZipDate::LEGACY_SETS,
        );
        let files = read_zip(&bytes).unwrap();
        assert_eq!(
            files.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(),
            ["x", "é", "x"]
        );
        assert_eq!(find_file(&files, "x"), Some(b"three".as_slice()));
    }

    #[test]
    fn detects_corruption() {
        let mut bytes = archive(&[("x", b"hello hello hello")], ZipDate::SETS);
        bytes[34] ^= 0xFF; // first byte of the deflate stream
        assert!(read_zip(&bytes).is_err());
    }

    /// Sets the declared uncompressed size of the first member in the central directory.
    fn declare_size(bytes: &mut [u8], size: u32) {
        let end = find_end_record(bytes).unwrap();
        let cd = u32_at(bytes, end + 16) as usize;
        bytes[cd + 24..cd + 28].copy_from_slice(&size.to_le_bytes());
    }

    #[test]
    fn members_never_inflate_past_their_declared_size_or_the_limits() {
        let zeros = vec![0u8; 3 << 20];
        let mut bomb = archive(&[("a/x.json", &zeros)], ZipDate::SETS);
        assert_eq!(read_zip(&bomb).unwrap()[0].data.len(), zeros.len());
        declare_size(&mut bomb, 10);
        let error = read_zip(&bomb).unwrap_err().to_string();
        assert_eq!(
            error,
            "archive exceeds the reader's limits: a/x.json inflates past its declared 10 bytes"
        );

        let small = ZipLimits {
            entries: 2,
            entry_bytes: 1 << 20,
            total_bytes: 5 << 20,
        };
        let one = archive(&[("a/x.json", b"{}")], ZipDate::SETS);
        assert_eq!(read_zip_with(&one, &small).unwrap().len(), 1);
        let big = archive(&[("a/x.json", &zeros)], ZipDate::SETS);
        assert!(matches!(
            read_zip_with(&big, &small),
            Err(ZipError::Limit(message)) if message.starts_with("a/x.json: 3145728 bytes")
        ));
        let many = archive(&[("a", b"1"), ("b", b"2"), ("c", b"3")], ZipDate::SETS);
        assert_eq!(
            read_zip_with(&many, &small).unwrap_err().to_string(),
            "archive exceeds the reader's limits: 3 members, more than 2"
        );
        // Members that share one stream still count in full.
        let mid = vec![0u8; 900 << 10];
        let shared = archive(&[("a", &mid), ("b", &mid), ("c", &mid)], ZipDate::SETS);
        let total = ZipLimits {
            entries: 8,
            total_bytes: 2 << 20,
            ..small
        };
        assert!(matches!(
            read_zip_with(&shared, &total),
            Err(ZipError::Limit(message)) if message.starts_with("c: ")
        ));
    }

    /// A one-member archive with a stored (method 0) entry.
    fn stored(name: &str, data: &[u8]) -> Vec<u8> {
        let mut crc = Crc::new();
        crc.update(data);
        let common = |out: &mut Vec<u8>| {
            for value in [20u16, 0, 0, 0, 0] {
                out.extend_from_slice(&value.to_le_bytes()); // version, flags, method, time, date
            }
            out.extend_from_slice(&crc.sum().to_le_bytes());
            for _ in 0..2 {
                out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            }
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
        };
        let mut out = LOCAL_HEADER.to_vec();
        common(&mut out);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(data);
        let cd = out.len() as u32;
        out.extend_from_slice(CENTRAL_HEADER);
        out.extend_from_slice(&20u16.to_le_bytes());
        common(&mut out);
        out.extend_from_slice(&[0; 14]); // comment length, disk, attributes, local offset 0
        out.extend_from_slice(name.as_bytes());
        let size = out.len() as u32 - cd;
        out.extend_from_slice(END_RECORD);
        out.extend_from_slice(&[0, 0, 0, 0, 1, 0, 1, 0]);
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&cd.to_le_bytes());
        out.extend_from_slice(&[0, 0]);
        out
    }

    #[test]
    fn stored_members_must_match_their_size() {
        let mut bytes = stored("x", b"hello");
        assert_eq!(read_zip(&bytes).unwrap()[0].data, b"hello");
        declare_size(&mut bytes, 4);
        assert_eq!(
            read_zip(&bytes).unwrap_err().to_string(),
            "invalid archive: x: CRC or size mismatch"
        );
    }

    #[test]
    fn dos_dates() {
        assert_eq!(ZipDate::LEGACY_SETS.dos(), 0x5D26);
        assert_eq!(ZipDate::ICON_LIBRARY.dos(), 0x5D27);
        assert_eq!(ZipDate::SETS.dos(), 0x5D28);
    }
}
