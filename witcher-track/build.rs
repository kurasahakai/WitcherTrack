use std::path::PathBuf;
use std::{env, fs};

fn out_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

/// Download english Tesseract trained data if it is not present.
pub fn download_trained_data() {
    const ENG_TRAINEDDATA_URL: &str =
        "https://github.com/tesseract-ocr/tessdata/raw/main/eng.traineddata";

    let trained_data_path = out_dir().join("eng.traineddata");

    if trained_data_path.exists() {
        return;
    }

    let response = ureq::get(ENG_TRAINEDDATA_URL).call().unwrap();
    let mut bytes = Vec::with_capacity(response.header("Content-Length").unwrap().parse().unwrap());
    response.into_reader().read_to_end(&mut bytes).unwrap();
    fs::write(trained_data_path, bytes).unwrap();
}

fn main() {
    download_trained_data();
    println!("cargo:rustc-link-lib=static=archive");
    println!("cargo:rustc-link-lib=User32");
    println!("cargo:rustc-link-lib=Crypt32");
}
