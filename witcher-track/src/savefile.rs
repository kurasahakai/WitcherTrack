use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};
use std::{env, fs, thread};

use anyhow::Result;
use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
    pub static ref QUEST_GUIDS: HashSet<String> = include_str!("../data/quest-guids.txt")
        .trim()
        .lines()
        .map(|s| s.to_ascii_uppercase())
        .collect();
    pub static ref MAP_PIN_TAGS: HashSet<String> = include_str!("../data/map-pin-tags.txt")
        .trim()
        .lines()
        .map(|s| s.to_ascii_uppercase())
        .collect();
}

#[derive(Deserialize, PartialEq, Eq, Hash)]
struct Quest {
    #[serde(rename = "Guid")]
    guid: String,
    #[serde(rename = "Status")]
    status: String,
}

#[derive(Deserialize)]
struct TrackerInfo {
    quests: Vec<Quest>,
    map_pin_tags: Vec<String>,
}

impl TrackerInfo {
    fn count_completion(self) -> (usize, usize) {
        let done_quests = self
            .quests
            .into_iter()
            .filter(|quest| quest.status == "Success")
            .map(|quest| quest.guid.to_ascii_uppercase())
            .filter(|guid| QUEST_GUIDS.contains(guid))
            .collect::<HashSet<_>>();

        let done_quests_count = done_quests.len();

        let done_pins = self.map_pin_tags.into_iter().collect::<HashSet<_>>();
        let done_pins_count = MAP_PIN_TAGS.intersection(&done_pins).count();

        (done_quests_count, done_pins_count)
    }
}

pub fn savefile_run() -> Result<()> {
    let savefile_directory = {
        let user_profile = PathBuf::from(env::var("USERPROFILE").unwrap());
        user_profile.join("Documents").join("The Witcher 3").join("gamesaves")
    };

    let exe_path = std::env::current_exe().unwrap();
    let helper_path = exe_path.parent().unwrap().join("save-helper.exe");

    println!("Savefile dir: {savefile_directory:?}");
    println!("Helper path:  {helper_path:?}");

    loop {
        let start = Instant::now();
        let last_save = fs::read_dir(&savefile_directory)
            .unwrap()
            .flatten()
            .filter(|f| {
                let is_file = f.metadata().unwrap().is_file();
                let is_sav = f.path().extension().map(|s| s == "sav").unwrap_or(false);

                is_file && is_sav
            })
            .max_by_key(|x| x.metadata().unwrap().modified().unwrap());

        println!("Last save:    {last_save:?}");

        if let Some(last_save) = last_save {
            let save_path = last_save.path();

            Command::new(&helper_path).arg(save_path).status().ok();

            let save_data: TrackerInfo =
                serde_json::from_str(&fs::read_to_string("./tw3trackerinfo.json")?)?;

            let (done_quests, done_pins) = save_data.count_completion();

            println!("Done quests: {done_quests}");
            println!("Done pins: {done_pins}");

            fs::write("tw3done_quests.txt", format!("{done_quests}"))?;
            fs::write("tw3done_pins.txt", format!("{done_pins}"))?;
        }
        println!("Took {:?}", start.elapsed());

        thread::sleep(Duration::from_millis(5000));
    }
}
