use std::io::Read;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use tracing::metadata::LevelFilter;
use witcher_track::data::{text_preprocess, tokenize, Action, GameRun};
use witcher_track::{download_trained_data, preprocess, screenshot, OcrReader};

fn run() -> Result<()> {
    download_trained_data()?;
    let ocr_reader = OcrReader::new()?;
    let mut game_run = GameRun::new()?;

    tracing_subscriber::fmt().with_max_level(LevelFilter::INFO).init();

    loop {
        let screenshot = screenshot::capture().and_then(|pic| unsafe { preprocess(pic) }).unwrap();
        let ocr_text =
            ocr_reader.get_ocr(&screenshot.into_cropped(0.4, 0.3)).map(text_preprocess)?;
        game_run.log(format!("Recognized: {ocr_text}"))?;
        match tokenize(ocr_text) {
            Some(Action::Quest(v)) => game_run.flag_quest(&v)?,
            Some(Action::Formula(v)) => game_run.flag_formula(&v)?,
            Some(Action::Diagram(v)) => game_run.flag_diagram(&v)?,
            None => (),
        }

        thread::sleep(Duration::from_millis(1000));
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e:#?}");
        let _ = std::io::stdin().read(&mut [0u8]).unwrap();
    }
}
