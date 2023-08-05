use anyhow::{anyhow, Result};
use std::{
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
    mem::{self, MaybeUninit},
    path::Path,
    slice,
};

pub fn read<R: Read, T: Sized>(mut reader: R) -> Result<T> {
    let mut buffer = MaybeUninit::uninit();
    let buffer_slice =
        unsafe { slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, mem::size_of::<T>()) };

    reader.read_exact(buffer_slice)?;
    Ok(unsafe { buffer.assume_init() })
}

struct Reader<R: Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> Reader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn read<T: Sized>(&mut self) -> Result<T> {
        read(&mut self.reader)
    }

    pub fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; count];
        self.reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn seek(&mut self, seek_from: SeekFrom) -> Result<u64> {
        Ok(self.reader.seek(seek_from)?)
    }
}

#[derive(Debug)]
#[repr(C)]
struct Lz4Header {
    count: i32,
    header_size: i32,
}

#[derive(Debug)]
#[repr(C)]
struct Lz4Chunk {
    compressed_size: i32,
    decompressed_size: i32,
    end_chunk_offset: i32,
}

pub fn read_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let mut fp = Reader::new(File::open(path.as_ref())?);

    let magic = fp.read::<[u8; 8]>()?;
    if &magic != b"SNFHFZLC" {
        Err(anyhow!("Wrong file magic {magic:?}"))?;
    }

    let header = fp.read::<Lz4Header>()?;
    println!("{header:?}");

    let mut chunks = Vec::with_capacity(header.count as usize);
    for _ in 0..header.count {
        chunks.push(fp.read::<Lz4Chunk>()?);
    }

    fp.reader.seek(SeekFrom::Start(header.header_size as u64))?;

    let mut all_buf = Vec::with_capacity(
        chunks
            .iter()
            .map(|c| c.decompressed_size as usize)
            .sum::<usize>()
            + header.header_size as usize,
    );
    all_buf.extend(vec![0u8; header.header_size as usize]);

    for chunk in chunks {
        let data = fp.read_bytes(chunk.compressed_size as usize)?;
        let buf = lz4_flex::block::decompress(&data[..], chunk.decompressed_size as usize)?;
        assert_eq!(buf.len(), chunk.decompressed_size as _);
        all_buf.extend(buf);
    }

    read_save_entry(&all_buf[..], &header)?;

    Ok(())
}

#[derive(Debug)]
#[repr(C, packed)]
struct W3Header {
    magic: [u8; 4],
    type_code1: i32,
    type_code2: i32,
    type_code3: i32,
}

#[derive(Debug)]
#[repr(C, packed)]
struct W3Footer {
    var_tbl_offset: i32,
    magic: [u8; 2],
}

#[derive(Debug)]
#[repr(C)]
struct VarIndex {
    offset: i32,
    size: i32,
}

fn read_save_entry(data: &[u8], header: &Lz4Header) -> Result<()> {
    let mut reader = Reader::new(Cursor::new(data));
    reader.seek(SeekFrom::Start(header.header_size as u64))?;

    // Read header.
    let header: W3Header = reader.read()?;
    assert_eq!(&header.magic, b"SAV3");

    // Read footer.
    reader.seek(SeekFrom::End(-6))?;
    let footer: W3Footer = reader.read()?;
    assert_eq!(&footer.magic, b"SE");

    // Read string offsets table.
    let string_tbl_footer_offset = footer.var_tbl_offset as u64 - 10;
    reader.seek(SeekFrom::Start(string_tbl_footer_offset))?;
    let nm_section_offset = reader.read::<i32>()?;
    let rb_section_offset = reader.read::<i32>()?;
    println!("{nm_section_offset:x} {rb_section_offset:x}");

    reader.seek(SeekFrom::Start(nm_section_offset as u64))?;
    assert_eq!(&reader.read::<[u8; 2]>()?, b"NM");
    reader.seek(SeekFrom::Start(rb_section_offset as u64))?;
    assert_eq!(&reader.read::<[u8; 2]>()?, b"RB");

    // Read main variable table.
    let count = reader.read::<i32>()?;
    let mut main_variable_indices = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let size = reader.read::<i16>()? as i32;
        let offset = reader.read::<i32>()?;
        main_variable_indices.push(VarIndex { size, offset });
    }

    // Read MANU string table.
    let string_tbl_offset = nm_section_offset as u64 + 2;

    reader.seek(SeekFrom::Start(string_tbl_offset))?;
    assert_eq!(&reader.read::<[u8; 4]>()?, b"MANU");

    let string_count = reader.read::<i32>()?;
    assert_eq!(reader.read::<i32>()?, 0);

    let mut string_table = Vec::new();
    string_table.push(String::new());

    for _ in 0..string_count {
        let str_size = reader.read::<u8>()? as usize;
        string_table.push(String::from_utf8(reader.read_bytes(str_size)?)?);
    }

    println!("Read {string_count:#?} strings");
    assert_eq!(reader.read::<i32>()?, 0);
    assert_eq!(&reader.read::<[u8; 4]>()?, b"ENOD");

    // Find CJournalManager
    for (idx, var_index) in main_variable_indices.iter().enumerate() {
        read_variable(&mut reader, var_index, &string_table)?;
    }

    // Read variable table.
    // reader.seek(SeekFrom::Start(footer.var_tbl_offset as u64))?;
    // let count = reader.read::<i32>()?;
    // let mut variable_indices = Vec::with_capacity(count as usize);
    // for _ in 0..count {
    //     let var_index = reader.read::<VarIndex>()?;
    //     variable_indices.push(var_index);
    // }
    //
    // println!("{:#?}", &variable_indices[..10]);
    //
    // for (idx, var_index) in variable_indices.iter().enumerate() {
    //     read_variable(&mut reader, var_index, &string_table)?;
    // }

    Ok(())
}

fn read_variable<T: Read + Seek>(
    reader: &mut Reader<T>,
    var_index: &VarIndex,
    string_table: &[String],
) -> Result<()> {
    reader.seek(SeekFrom::Start(var_index.offset as u64))?;

    let magic: [u8; 2] = reader.read()?;

    match &magic {
        b"BS" => {
            let var = read_bs(reader, var_index, string_table)?;
            if var.name == "CJournalManager" {
                println!("{var:?}");
            }
            // println!("{var:?}");
        }
        b"VL" => {
            let var = read_vl(reader, var_index, string_table)?;
            // println!("{var:?}");
        }
        b"SX" => {}
        b"SB" => {}
        b"RO" => {}
        b"SS" => {}
        b"OP" => {}
        b"BL" => {}
        b"AV" => {}
        b"PO" => {}
        _ => {
            // println!("Unrecognized magic: {magic:?}");
        }
    };

    Ok(())
}

#[derive(Debug)]
struct W3Bs {
    name: String,
    size: i32,
    offset: i32,
}

fn read_bs<T: Read + Seek>(
    reader: &mut Reader<T>,
    var_index: &VarIndex,
    string_table: &[String],
) -> Result<W3Bs> {
    let string_index: u16 = reader.read()?;
    Ok(W3Bs {
        name: string_table[string_index as usize].clone(),
        size: 4,
        offset: var_index.offset,
    })
}

#[derive(Debug)]
struct W3Vl {
    name: String,
    ty: String,
    value: String,
    size: i32,
    offset: i32,
}

fn read_vl<T: Read + Seek>(
    reader: &mut Reader<T>,
    var_index: &VarIndex,
    string_table: &[String],
) -> Result<W3Vl> {
    let name_index: u16 = reader.read()?;
    let type_index: u16 = reader.read()?;
    let name = string_table[name_index as usize].clone();
    let ty = string_table[type_index as usize].clone();
    let value = read_value(reader, &ty)?;
    Ok(W3Vl {
        name,
        ty,
        value,
        size: 4,
        offset: var_index.offset,
    })
}

fn read_value<T: Read + Seek>(reader: &mut Reader<T>, type_name: &str) -> Result<String> {
    let value = if type_name.starts_with("array:") {
        String::from("Array")
    } else if type_name.starts_with("handle:") {
        String::from("Handle")
    } else if type_name.starts_with("soft:") {
        String::from("Soft")
    } else {
        match type_name {
            "Uint8" => reader.read::<u8>()?.to_string(),
            "Uint16" => reader.read::<u16>()?.to_string(),
            "Int16" => reader.read::<i16>()?.to_string(),
            "Uint32" => reader.read::<u32>()?.to_string(),
            "Int32" => reader.read::<i32>()?.to_string(),
            "Uint64" => reader.read::<i64>()?.to_string(),
            "LocalizedString" => reader.read::<i32>()?.to_string(),
            "Float" => reader.read::<f32>()?.to_string(),
            "Double" => reader.read::<f64>()?.to_string(),
            "Bool" => (reader.read::<u8>()? == 1).to_string(),
            _ => {
                // println!("Unimplemented {type_name}");
                String::new()
            }
        }
    };

    Ok(value)
}
