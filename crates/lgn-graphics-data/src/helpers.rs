#![allow(clippy::trivially_copy_pass_by_ref, dead_code)]

use std::io;

use lgn_math::{Vec2, Vec4};

pub fn write_usize(writer: &mut dyn io::Write, v: usize) -> io::Result<usize> {
    let bytes = v.to_ne_bytes();
    writer.write_all(&bytes)?;
    Ok(bytes.len())
}

pub fn read_usize(reader: &mut dyn io::Read) -> io::Result<usize> {
    let mut byte_size = 0usize.to_ne_bytes();
    reader.read_exact(&mut byte_size)?;
    Ok(usize::from_ne_bytes(byte_size))
}

pub fn write_usize_and_buffer(writer: &mut dyn io::Write, v: &[u8]) -> io::Result<usize> {
    let written = write_usize(writer, v.len())?;
    writer.write_all(v)?;
    Ok(written + v.len())
}

pub fn read_usize_and_buffer(reader: &mut dyn io::Read) -> io::Result<Vec<u8>> {
    let size = read_usize(reader)?;
    let mut bytes = vec![0; size];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

pub fn write_vec4(writer: &mut dyn io::Write, v: &Vec4) -> io::Result<usize> {
    let bytes = v
        .to_array()
        .iter()
        .flat_map(|v| v.to_ne_bytes())
        .collect::<Vec<u8>>();
    writer.write_all(&bytes)?;
    Ok(bytes.len())
}

pub fn read_vec4(reader: &mut dyn io::Read) -> io::Result<Vec4> {
    let mut bytes = [0; std::mem::size_of::<Vec4>()];
    reader.read_exact(&mut bytes)?;
    Ok(Vec4::new(
        f32::from_ne_bytes(bytes[0..4].try_into().unwrap()),
        f32::from_ne_bytes(bytes[4..8].try_into().unwrap()),
        f32::from_ne_bytes(bytes[8..12].try_into().unwrap()),
        f32::from_ne_bytes(bytes[12..].try_into().unwrap()),
    ))
}

pub fn write_vec_vec4(writer: &mut dyn io::Write, v: &[Vec4]) -> io::Result<usize> {
    let mut written = write_usize(writer, v.len())?;
    for vector in v {
        written += write_vec4(writer, vector)?;
    }
    Ok(written)
}

pub fn read_vec_vec4(reader: &mut dyn io::Read) -> io::Result<Vec<Vec4>> {
    let length = read_usize(reader)?;
    let mut v = Vec::new();
    for _i in 0..length {
        v.push(read_vec4(reader)?);
    }
    Ok(v)
}

pub fn write_vec2(writer: &mut dyn io::Write, v: &Vec2) -> io::Result<usize> {
    let bytes = v
        .to_array()
        .iter()
        .flat_map(|v| v.to_ne_bytes())
        .collect::<Vec<u8>>();
    writer.write_all(&bytes)?;
    Ok(bytes.len())
}

pub fn read_vec2(reader: &mut dyn io::Read) -> io::Result<Vec2> {
    let mut bytes = [0; std::mem::size_of::<Vec2>()];
    reader.read_exact(&mut bytes)?;
    Ok(Vec2::new(
        f32::from_ne_bytes(bytes[0..4].try_into().unwrap()),
        f32::from_ne_bytes(bytes[4..8].try_into().unwrap()),
    ))
}

pub fn write_vec_vec2(writer: &mut dyn io::Write, v: &[Vec2]) -> io::Result<usize> {
    let mut written = write_usize(writer, v.len())?;
    for vector in v {
        written += write_vec2(writer, vector)?;
    }
    Ok(written)
}

pub fn read_vec_vec2(reader: &mut dyn io::Read) -> io::Result<Vec<Vec2>> {
    let length = read_usize(reader)?;
    let mut v = Vec::new();
    for _i in 0..length {
        v.push(read_vec2(reader)?);
    }
    Ok(v)
}

pub fn write_u32(writer: &mut dyn io::Write, v: &u32) -> io::Result<usize> {
    writer.write(&v.to_ne_bytes())
}

pub fn read_u32(reader: &mut dyn io::Read) -> io::Result<u32> {
    let mut byte_size = [0; 4];
    reader.read_exact(&mut byte_size)?;
    Ok(u32::from_ne_bytes(byte_size))
}

pub fn write_vec_u32(writer: &mut dyn io::Write, v: &[u32]) -> io::Result<usize> {
    let written = write_usize(writer, v.len())?;
    for value in v {
        write_u32(writer, value)?;
    }
    Ok(written + 4 * v.len())
}

pub fn read_vec_u32(reader: &mut dyn io::Read) -> io::Result<Vec<u32>> {
    let length = read_usize(reader)?;
    let mut v = Vec::new();
    for _i in 0..length {
        v.push(read_u32(reader)?);
    }
    Ok(v)
}
