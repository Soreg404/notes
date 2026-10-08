use std::{
    path::{PathBuf},
};
use crate::{
    exit_process::proc_exit,
    helpers,
};

pub fn edit_note(n: usize) {
    let t = chrono::Local::now();
    let t_fmt = t.format("%Y-%m-%d %H:%M:%S %Z");

    let note_num = format!("{:08}", n);

    let filename = &PathBuf::from(format!("data/objects-store/{note_num}.md"));

    let note = match std::fs::read_to_string(filename) {
        Ok(v) => v,
        Err(ioerr) => {
            proc_exit(Err(format!("\x1b[91mnote missing!!\x1b[0m aborting (io error: {ioerr:?})")));
        }
    };

    let note_draft_path = &PathBuf::from(format!("data/drafts/{note_num}-edit.draft.md"));

    let last_lf = if !note.ends_with("\n") { "\n" } else { "" };

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

        std::fs::write(
            filename,
            note
        ).unwrap();

        std::fs::remove_file(note_draft_path)
            .unwrap();

        helpers::git_commit(&note_num, true);
    }
}

