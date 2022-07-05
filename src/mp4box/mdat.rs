use serde::Serialize;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::mp4box::*;
use crate::mp4box::{dinf::DinfBox, smhd::SmhdBox, stbl::StblBox, vmhd::VmhdBox};

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct MdatBox {
    #[serde(skip_serializing)]
    pub data: Vec<u8>,
}

impl MdatBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::MdatBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + self.data.len() as u64
    }
}

impl Mp4Box for MdatBox {
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

impl<R: Read + Seek> ReadBox<&mut R> for MdatBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let mut data = vec![0;size as usize];
        reader.read_exact(&mut data[0..size as usize])?;
        Ok(MdatBox {
            data
        })
    }
}

impl<W: Write> WriteBox<&mut W> for MdatBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;
        writer.write(&self.data)?;
        Ok(size)
    }
}
