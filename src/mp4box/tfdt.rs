use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TfdtBox {
    pub version: u8,
    pub flags: u32,
    pub base_media_decode_time: u64,
}

impl Default for TfdtBox {
    fn default() -> Self {
        TfdtBox {
            version: 0,
            flags: 0,
            base_media_decode_time: 0
        }
    }
}

impl TfdtBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::TfdtBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;
        size += match self.version {
            1 => 8,
            _ => 4,
        };
        size
    }
}

impl Mp4Box for TfdtBox {
    fn box_type(&self) -> BoxType {
        self.get_type()
    }

    fn box_size(&self) -> u64 {
        self.get_size()
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = String::new();
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for TfdtBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;
        let base_media_decode_time = match version {
            1 => reader.read_u64::<BigEndian>()?,
            _ => reader.read_u32::<BigEndian>()? as u64
        };
        skip_bytes_to(reader, start + size)?;
        Ok(TfdtBox {
            version,
            flags,
            base_media_decode_time
        })
    }
}

impl<W: Write> WriteBox<&mut W> for TfdtBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        write_box_header_ext(writer, self.version, self.flags)?;
        match self.version {
            1 => writer.write_u64::<BigEndian>(self.base_media_decode_time)?,
            _ => writer.write_u32::<BigEndian>(self.base_media_decode_time as _)?
        };
        Ok(size)
    }
}
