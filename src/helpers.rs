use std::{
    path::Path,
};

pub fn create_editor_proc(input_edit: &Path) -> std::process::Child {
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

pub fn strip_html_comments(note: &String) -> String {
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

pub fn git_commit(note_num: &str, edit: bool) {
    std::process::Command::new("git")
        .current_dir("data")
        .args(&["add", "."])
        .status()
        .unwrap();

    let msg = &format!("note#{note_num} {}", if edit { "edited" } else { "added" });

    std::process::Command::new("git")
        .current_dir("data")
        .args(&["commit", "-m", msg, "-mautocommit"])
        .status()
        .unwrap();
}
