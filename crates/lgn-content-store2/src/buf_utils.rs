use byteorder::{ByteOrder, ReadBytesExt};
use std::io::{Read, Write};

pub(crate) fn write_prefixed_size(mut w: impl Write, size: u64) -> std::io::Result<()> {
    let mut size_buf = [0; 8];
    byteorder::NetworkEndian::write_u64(&mut size_buf, size);

    let idx = size_buf
        .iter()
        .position(|&b| b != 0)
        .unwrap_or(size_buf.len() - 1);

    let size_len: u8 = (size_buf.len() - idx).try_into().unwrap();

    w.write_all(&[size_len])?;
    w.write_all(&size_buf[idx..])
}

pub(crate) fn read_prefixed_size(mut r: impl Read) -> std::io::Result<Option<u64>> {
    let size_len = r.read_u8()?;

    if size_len == 0 {
        return Ok(None);
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

    Ok(Some(size))
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
