use std::{
    path::{Path, PathBuf},
};
use crate::{
    exit_process::proc_exit,
    store,
    helpers,
};

pub fn create_note() {
    let t = chrono::Local::now();
    let t_fmt = t.format("%Y-%m-%d %H:%M:%S %Z");

    let note_num = format!("{:05}", store::get_counter());

    let note_draft_path = &PathBuf::from(format!("data/drafts/{note_num}.draft.md"));

    std::fs::write(
        note_draft_path,
        format!("\n\n\
            <!--\n\
                Adding new note.\n\
                Comments are ignored and stripped.\n\
                Empty note aborts operation.\n\
                time: {t_fmt}\n\
                -->").as_bytes(),
    )
        .unwrap();

    helpers::create_editor_proc(note_draft_path)
        .wait()
        .unwrap();
    let note = match std::fs::read_to_string(
        note_draft_path
    ) {
        Ok(v) => v,
        Err(ioerr) => {
            proc_exit(Err(format!("draft missing, aborting (io error: {ioerr:?})")));
        }
    };

    let note = helpers::strip_html_comments(&note);

    let mut note_lines = Vec::new();
    let mut is_empty_note = true;
    for line in note.lines() {
        if is_empty_note && !line.trim().is_empty() {
            is_empty_note = false;
        }
        note_lines.push(line.trim_end());
    }

    if is_empty_note {
        std::fs::remove_file(note_draft_path)
            .unwrap();
        proc_exit(Err(String::from("Empty note, aborting.")));
    } else {
        let note = note_lines.join("\n");

        let note_name = {
            let mut tmp = PathBuf::from(note_draft_path.file_stem().unwrap());
            tmp.set_extension("md");
            tmp
        };
        std::fs::write(
            Path::new("data/objects-store/").join(note_name),
            note
        )
            .unwrap();

        std::fs::remove_file(note_draft_path)
            .unwrap();

        helpers::git_commit(&note_num, false);
    }
}

