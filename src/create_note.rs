use std::{
    path::{Path, PathBuf},
};
use crate::{
    exit_process::proc_exit,
    store,
};

pub fn create_note() {
    let t = chrono::Local::now();
    let t_fmt = t.format("%Y-%m-%d %H:%M:%S %Z");

    let note_draft_path = &{
        let note_num = store::get_counter();
        let note_draft_name = format!("{note_num:05}.draft.md");
        Path::new("data/drafts/").join(note_draft_name)
    };

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

    create_editor_proc(note_draft_path)
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

    let note = strip_html_comments(&note);

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
    }
}

fn create_editor_proc(input_edit: &Path) -> std::process::Child {
    std::process::Command::new("cmd")
        .arg("/c")
        .arg("vim")
        .arg(input_edit)
        .arg("-cset nofixeol")
        .arg("-cset filetype=markdown")
        .arg("+0")
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .spawn()
        .unwrap()
}

fn strip_html_comments(note: &String) -> String {
    let mut tmp = Vec::<u8>::new();
    let note = note.as_bytes();
    let mut i = 0;
    let mut in_comment = false;
    while i < note.len() {
        let s = &note[i..];
        if in_comment && s.starts_with(b"-->") {
            in_comment = false;
            i += 3;
            continue;
        } else if !in_comment && s.starts_with(b"<!--") {
            in_comment = true;
            i += 4;
            continue;
        } else if !in_comment {
            tmp.push(note[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&tmp).to_string()
}
