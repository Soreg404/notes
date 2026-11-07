use std::{
    process::exit,
    path::Path,
    fs::read_dir,
};

use chrono::prelude::*;

use crate::{
    store,
    prompt::prompt_yn,
};

const CHECK_FILE: &str = "data/metadata/patch-dates-inobj";

pub fn is_applied() -> bool {
    if Path::new(CHECK_FILE).exists() {
        true
    } else {
        false
    }
}

pub fn patch() {
    println!("\x1b[93mpatch: write filetime dates to all existing notes.\x1b[0m");

    if !prompt_yn() { exit(1); };

    for p in read_dir(store::notes_path()).unwrap() {
        let p = &p.unwrap().path();
        println!("doing path: {p:?}");
        let secs = std::fs::metadata(p).unwrap().modified().unwrap()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let date = DateTime::<Utc>::from_timestamp_secs(
            secs as i64).unwrap();

        let mut note = match std::fs::read_to_string(p){
            Err(ioerr) => {
                eprintln!("failed to read note {p:?}: {ioerr:?}");
                continue;
            }
            Ok(v) => v
        };



        note = crate::helpers::set_note_date(note, date);
        println!("  date: {}", DateTime::<FixedOffset>::from(date).to_rfc3339());

        std::fs::write(p, note).unwrap();
    }

    std::fs::write(CHECK_FILE, b"applied patch: note dates in note objects")
        .unwrap();
}
