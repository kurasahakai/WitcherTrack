use std::fs::File;
use std::io::{self, Result};
use std::path::Path;

use lz4::Decoder;
use witcher_save::read_file;

fn decompress(source: &Path, destination: &Path) -> Result<()> {
    println!(
        "Decompressing: {} -> {}",
        source.display(),
        destination.display()
    );

    let input_file = File::open(source)?;
    let mut decoder = Decoder::new(input_file)?;
    let mut output_file = File::create(destination)?;
    io::copy(&mut decoder, &mut output_file)?;

    Ok(())
}

fn main() {
    // decompress(Path::new("QuickSave.sav"), Path::new("foo.wtf")).unwrap();
    read_file("QuickSave.sav").unwrap();
}
