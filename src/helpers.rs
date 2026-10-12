use std::{
    str::FromStr,
    path::Path,
};

use chrono::{
    DateTime,
    FixedOffset,
    Utc,
};

pub fn get_note_date(note: &str) -> DateTime<Utc> {
    match note.lines().next()
    .map(|v| DateTime::<Utc>::from_str(v)) {
        Some(Ok(v)) => v.to_utc(),
        None => panic!("\x1b[91mnote doesn't have date\x1b[0m"),
        Some(Err(e)) => panic!("\x1b[91mnote has invalid date: {e:?}\x1b[0m"),
    }
}

pub fn set_note_date(note: &mut String, date: DateTime<Utc>) {
    let date_str = DateTime::<FixedOffset>::from(date).to_rfc3339();
    *note = format!("{date_str}\n{note}");
}

/// todo: remove in next patch
pub fn patch_create_note_date_from_ftime(
    mut note: String,
    p: &Path,
) -> String {
    let secs = std::fs::metadata(p).unwrap().modified().unwrap()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let date = DateTime::<Utc>::from_timestamp_secs(
        secs as i64).unwrap();
    set_note_date(&mut note, date);
    note
}

pub fn get_note_title(note: &str) -> &str {
    note.lines()
        .skip(1).take(1).next()
        .map(|v| v.trim())
        .expect("empty note is not allowed")
}

pub fn date_relative_str(datetime: DateTime<Utc>) -> String {
    let now = Utc::now();
    let time_diff = now - datetime;
    if time_diff.num_minutes() < 10 {
        "just now".to_string()
    } else if now.date_naive() == datetime.date_naive() {
        "today".to_string()
    } else if time_diff.num_days() == 0 {
        "yesterday".to_string()
    }
    else if time_diff.num_days() < 31 {
        format!("{} days ago", time_diff.num_days() + 1)
    } else if time_diff.num_days() < 365 {
        format!("{} month(s) {} day(s) ago", time_diff.num_days() / 30,
            time_diff.num_days() % 30)
    } else {
        format!("{} year(s) {} month(s) {} day(s) ago",
        time_diff.num_days() / 365, (time_diff.num_days() % 365) / 30,
        ((time_diff.num_days() % 365) / 30) % 30)
    }
}

pub fn create_editor_proc(input_edit: &Path) -> std::process::Child {
    // todo: change to vim maybe??
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
