use std::io::Read;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use tracing::metadata::LevelFilter;
use witcher_track::data::{parse_action, slugify, Action};
use witcher_track::db::GameRun;
use witcher_track::picture::preprocess;
use witcher_track::{screenshot, OcrReader, CROP_RANGE};

fn run() -> Result<()> {
    let ocr_reader = OcrReader::new()?;
    let mut game_run = GameRun::new()?;

    tracing_subscriber::fmt().with_max_level(LevelFilter::INFO).init();

    loop {
        let start = Instant::now();
        let screenshot = screenshot::capture().and_then(|pic| unsafe { preprocess(pic) })?;
        let cropped = screenshot.into_cropped(CROP_RANGE.0, CROP_RANGE.1)?;
        let ocr_text = ocr_reader.get_ocr(&cropped).map(slugify)?;
        if !ocr_text.trim().is_empty() {
            game_run.log("RECOGNIZED", &ocr_text)?;
        }
        match parse_action(ocr_text) {
            Some(Action::Quest(v)) => game_run.flag_quest(&v)?,
            Some(Action::Formula(v)) => game_run.flag_formula(&v)?,
            Some(Action::Diagram(v)) => game_run.flag_diagram(&v)?,
            None => (),
        }
        game_run.timing(start.elapsed())?;

        thread::sleep(Duration::from_millis(100));
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Errored out: {e:#?}");
        let _ = std::io::stdin().read(&mut [0u8]).unwrap();
    }
}
