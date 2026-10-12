use std::{
    path::{Path, PathBuf},
};
use crate::{
    exit_process::proc_exit,
    store,
    helpers,
};

use chrono::{
    DateTime,
    FixedOffset,
    Local,
};

pub fn create_note() {
    let note_num = store::get_counter_string();
    let note_draft_path = &PathBuf::from(format!("data/drafts/{note_num}.draft.md"));

    let tmp_date = DateTime::<FixedOffset>::from(Local::now().to_utc()).to_rfc3339();

    std::fs::write(
        note_draft_path,
        format!("\n\n\
            <!--\n\
                Adding new note.\n\
                Comments are ignored and stripped.\n\
                Empty note aborts operation.\n\
                time: {tmp_date}\n\
                -->").as_bytes(),
    ).unwrap();

    helpers::create_editor_proc(note_draft_path)
        .wait()
        .unwrap();
    let note = match std::fs::read_to_string(note_draft_path) {
        Ok(v) => v,
        Err(ioerr) => {
            proc_exit(Err(format!("draft missing, aborting (io error: {ioerr:?})")));
        }
    };

    let note = match note_from_draft(note) {
        Some(v) => v,
        None => {
            std::fs::remove_file(note_draft_path)
                .unwrap();
            proc_exit(Err(String::from("Empty note, aborting.")));
        }
    };

    let note_name = format!("{note_num}.md");
    std::fs::write(
        Path::new("data/objects-store/").join(note_name),
        note
    ).unwrap();

    std::fs::remove_file(note_draft_path)
        .unwrap();

    helpers::git_commit(&note_num, false);
}

pub fn note_from_draft(mut note: String) -> Option<String> {
    let note = helpers::strip_html_comments(&mut note);

    let mut note_lines = Vec::new();
    let mut is_empty_note = true;
    for line in note.lines() {
        if is_empty_note && !line.trim().is_empty() {
            is_empty_note = false;
        }
        note_lines.push(line.trim_end());
    }
    if is_empty_note {
        return None;
    }

    let mut note = note_lines.join("\n");

    let date = chrono::Local::now().to_utc();
    helpers::set_note_date(&mut note, date);

    Some(note)
}
