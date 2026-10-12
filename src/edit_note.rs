use std::{
    path::{PathBuf},
};
use crate::{
    exit_process::proc_exit,
    helpers,
};

use chrono::{
    DateTime,
    FixedOffset,
    Utc,
};

pub fn edit_note(note_num: &str) {
    let filename = &PathBuf::from(format!("data/objects-store/{note_num}.md"));

    let note = match std::fs::read_to_string(filename) {
        Ok(v) => v,
        Err(ioerr) => {
            proc_exit(Err(format!("\x1b[91mnote missing!!\x1b[0m aborting (io error: {ioerr:?})")));
        }
    };
    let note = note.lines().skip(1).join("\n");
    let last_lf = if !note.ends_with("\n") { "\n" } else { "" };

    let note_draft_path = &PathBuf::from(format!("data/drafts/{note_num}-edit.draft.md"));

    std::fs::write(
        note_draft_path,
        format!(
            "{note}\
            {last_lf}\
            <!--\n\
                Adding new note.\n\
                Comments are ignored and stripped.\n\
                Empty note aborts operation.\n\
                time: {t_fmt}\n\
                -->").as_bytes(),
    ).unwrap();

    helpers::create_editor_proc(note_draft_path)
        .wait()
        .unwrap();
    let note = match std::fs::read_to_string(note_draft_path) {
        Ok(v) => v,
        Err(ioerr) => {
            proc_exit(Err(format!("\x1b[91mnote missing!!\x1b[0m aborting (io error: {ioerr:?})")));
        }
    };

    let note = match crate::create_note::note_from_draft(note) {
        Some(v) => v,
        None => {
            std::fs::remove_file(note_draft_path)
                .unwrap();
            proc_exit(Err(String::from("Empty note, aborting.")));
        }
    };

    std::fs::write(
        filename,
        note
    ).unwrap();

    std::fs::remove_file(note_draft_path)
        .unwrap();

    helpers::git_commit(&note_num, true);
}

