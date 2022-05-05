use byteorder::{ByteOrder, ReadBytesExt};
use std::io::{Read, Write};

/// Write a prefixed size to the specified writer.
///
/// The `high_bits` parameter specifies the value of the unused first 4 bits of
/// the size prefix length. This can be used to convey additional information
/// about the size or its surrounding data. Cannot exceed 15.
pub(crate) fn write_prefixed_size(
    mut w: impl Write,
    size: u64,
    high_bits: u8,
) -> std::io::Result<()> {
    let mut size_buf = [0; 8];
    byteorder::NetworkEndian::write_u64(&mut size_buf, size);

    let idx = size_buf
        .iter()
        .position(|&b| b != 0)
        .unwrap_or(size_buf.len());

    let size_len: u8 = (size_buf.len() - idx).try_into().unwrap();

    // The size should never take more than 16 bytes, which is a maximum of 8^16
    // or the maximum value of a u128.
    if size_len > 16 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("size_len is too large ({} > 16)", size_len),
        ));
    }

    assert!(high_bits <= 16, "high_bits should be stricly less than 16");

    let size_len = size_len | high_bits << 4;

    w.write_all(&[size_len])?;
    w.write_all(&size_buf[idx..])
}

/// Read a prefixed size from the specified reader.
///
/// Returns `None` if the size prefix is not present.
///
/// The second element of the returned tuple is the high bits that were set
/// during the call to `write_prefixed_size`.
pub(crate) fn read_prefixed_size(mut r: impl Read) -> std::io::Result<(Option<u64>, u8)> {
    let size_len = r.read_u8()?;

    let high_bits = size_len >> 4;
    let size_len = size_len & 0x0f;

    if size_len == 0 {
        return Ok((None, high_bits));
    }

    let mut size_buf = [0; 8];

    if size_len as usize > size_buf.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid size length",
        ));
    }

    r.read_exact(&mut size_buf[8 - size_len as usize..])?;
    let size = byteorder::NetworkEndian::read_u64(&size_buf);

    Ok((Some(size), high_bits))
}

pub(crate) fn get_size_len(size: u64) -> usize {
    let mut size_buf = [0; 8];
    byteorder::NetworkEndian::write_u64(&mut size_buf, size);

    let idx = size_buf
        .iter()
        .position(|&b| b != 0)
        .unwrap_or(size_buf.len() - 1);

    size_buf.len() - idx
}
