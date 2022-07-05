use serde::Serialize;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use crate::mp4box::*;
use crate::mp4box::{tfhd::TfhdBox, trun::TrunBox};
use crate::tfdt::TfdtBox;

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct TrafBox {
    pub tfhd: TfhdBox,
    pub tfdt: Option<TfdtBox>,
    pub trun: Option<TrunBox>,
}

impl TrafBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::TrafBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        size += self.tfhd.box_size();
        if let Some(ref tfdt) = self.tfdt {
            size += tfdt.box_size();
        }
        if let Some(ref trun) = self.trun {
            size += trun.box_size();
        }
        size
    }
}

impl Mp4Box for TrafBox {
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

impl<R: Read + Seek> ReadBox<&mut R> for TrafBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut tfhd = None;
        let mut trun = None;
        let mut tfdt = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;

            match name {
                BoxType::TfhdBox => {
                    tfhd = Some(TfhdBox::read_box(reader, s)?);
                }
                BoxType::TrunBox => {
                    trun = Some(TrunBox::read_box(reader, s)?);
                }
                BoxType::TfdtBox => {
                    tfdt = Some(TfdtBox::read_box(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        if tfhd.is_none() {
            return Err(Error::BoxNotFound(BoxType::TfhdBox));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(TrafBox {
            tfhd: tfhd.unwrap(),
            trun,
            tfdt
        })
    }
}

impl<W: Write> WriteBox<&mut W> for TrafBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        self.tfhd.write_box(writer)?;
        if let Some(tfdt) = &self.tfdt {
            tfdt.write_box(writer)?;
        }
        if let Some(trun) = &self.trun {
            trun.write_box(writer)?;
        }
        Ok(size)
    }
}



#[test]
fn test_traf_same_size() {
    let src_box = TrafBox {
        tfhd: Default::default(),
        tfdt: Some(TfdtBox::default()),
        trun: Some(TrunBox::default())
    } ;
    let mut buf = Vec::new();
    src_box.write_box(&mut buf).unwrap();
    assert_eq!(buf.len(), src_box.box_size() as usize);

    let mut reader = Cursor::new(&buf);
    let header = BoxHeader::read(&mut reader).unwrap();
    assert_eq!(header.name, BoxType::TrafBox);
    assert_eq!(src_box.box_size(), header.size);

    let dst_box = TrafBox::read_box(&mut reader, header.size).unwrap();
    assert_eq!(src_box, dst_box);
}
