use std::collections::HashSet;
use std::ffi::{c_void, CString};
use std::path::PathBuf;
use std::time::Duration;
use std::{env, fs, thread};

use anyhow::Result;
use lazy_static::lazy_static;
use libloading::{Library, Symbol};
use serde::Deserialize;

lazy_static! {
    pub static ref QUEST_GUIDS: HashSet<String> =
        include_str!("../../save-helper/data/quest-guids.txt")
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
    map_pin_tag: Vec<String>,
}

impl TrackerInfo {
    fn count_done_quests(&self) -> usize {
        let done_quests = self
            .quests
            .iter()
            .filter(|quest| quest.status == "Success")
            .map(|quest| quest.guid.to_ascii_uppercase())
            .filter(|guid| QUEST_GUIDS.contains(guid))
            .collect::<HashSet<_>>();

        done_quests.len()
    }

    fn count_done_pins(&self) -> usize {
        0
    }
}

pub fn savefile_run() -> Result<()> {
    let savefile_directory = {
        let user_profile = PathBuf::from(env::var("USERPROFILE").unwrap());
        user_profile.join("Documents").join("The Witcher 3").join("gamesaves")
    };

    let exe_path = std::env::current_exe().unwrap();
    let lib_path = exe_path.parent().unwrap().join("WitcherSaveTracker.dll");

    println!("Savefile dir: {savefile_directory:?}");
    println!("Exe path:     {exe_path:?}");
    println!("Lib path:     {lib_path:?}");

    let lib = unsafe { Library::new(lib_path)? };
    println!("Lib loaded");
    let export_save: Symbol<unsafe extern "C" fn(*const i8) -> ()> =
        unsafe { lib.get(b"export_save")? };
    println!("Export loaded");

    loop {
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
            let save_path = PathBuf::from("../witcher-save-cs/data/QuickSave.sav");
            let save_path_cstr = CString::new(save_path.to_str().unwrap().as_bytes()).unwrap();
            println!("{:?}", save_path);
            unsafe { export_save(save_path_cstr.as_ptr()) };
            let save_data: TrackerInfo =
                serde_json::from_str(&fs::read_to_string("./tw3trackerinfo.json")?)?;

            let done_quests = save_data.count_done_quests();
            let done_pins = save_data.count_done_pins();

            println!("Done quests: {done_quests}");
            println!("Done pins: {done_pins}");
        }

        thread::sleep(Duration::from_millis(5000));
        break;
    }

    Ok(())
}
