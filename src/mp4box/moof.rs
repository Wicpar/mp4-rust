use serde::Serialize;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use crate::mp4box::*;
use crate::mp4box::{mfhd::MfhdBox, traf::TrafBox};
use crate::tfdt::TfdtBox;
use crate::tfhd::TfhdBox;
use crate::trun::TrunBox;

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct MoofBox {
    pub mfhd: MfhdBox,

    #[serde(rename = "traf")]
    pub trafs: Vec<TrafBox>,
}

impl MoofBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::MoofBox
    }

    pub fn get_size(&self) -> u64 {
        let mut size = HEADER_SIZE + self.mfhd.box_size();
        for traf in self.trafs.iter() {
            size += traf.box_size();
        }
        size
    }
}

impl Mp4Box for MoofBox {
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
        let s = format!("trafs={}", self.trafs.len());
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for MoofBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut mfhd = None;
        let mut trafs = Vec::new();

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = BoxHeader::read(reader)?;
            let BoxHeader { name, size: s } = header;

            match name {
                BoxType::MfhdBox => {
                    mfhd = Some(MfhdBox::read_box(reader, s)?);
                }
                BoxType::TrafBox => {
                    let traf = TrafBox::read_box(reader, s)?;
                    trafs.push(traf);
                }
                _ => {
                    // XXX warn!()
                    skip_box(reader, s)?;
                }
            }
            current = reader.seek(SeekFrom::Current(0))?;
        }

        if mfhd.is_none() {
            return Err(Error::BoxNotFound(BoxType::MfhdBox));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(MoofBox {
            mfhd: mfhd.unwrap(),
            trafs,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for MoofBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        self.mfhd.write_box(writer)?;
        for traf in self.trafs.iter() {
            traf.write_box(writer)?;
        }
        Ok(size)
    }
}



#[test]
fn test_moof_same_size() {
    let src_box = MoofBox {
        mfhd: MfhdBox {
            version: 0,
            flags: 0,
            sequence_number: 1
        },
        trafs: vec![TrafBox {
            tfhd: TfhdBox {
                version: 0,
                flags: 0x020000, // offset relative to moof flag
                track_id: 1,
                base_data_offset: 0 // offset addition
            },
            tfdt: Some(TfdtBox {
                version: 0,
                flags: 0,
                base_media_decode_time: 1
            }),
            trun: Some(TrunBox {
                version: 0,
                flags: TrunBox::FLAG_SAMPLE_DURATION | TrunBox::FLAG_SAMPLE_SIZE,
                sample_count: 1,
                data_offset: None,
                first_sample_flags: None,
                sample_durations: vec![1],
                sample_sizes: vec![1],
                sample_flags: vec![],
                sample_cts: vec![]
            })
        }]
    };
    let mut buf = Vec::new();
    src_box.write_box(&mut buf).unwrap();
    assert_eq!(buf.len(), src_box.box_size() as usize);

    let mut reader = Cursor::new(&buf);
    let header = BoxHeader::read(&mut reader).unwrap();
    assert_eq!(header.name, BoxType::MoofBox);
    assert_eq!(src_box.box_size(), header.size);

    let dst_box = MoofBox::read_box(&mut reader, header.size).unwrap();
    assert_eq!(src_box, dst_box);
}
