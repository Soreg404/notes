use std::{
    str::FromStr,
    path::Path,
};

use chrono::prelude::*;

pub fn get_strip_note_date(note: &mut String) -> DateTime<Utc> {
    let line1_len;
    let date_str = match note.lines().next() {
        None => panic!("\x1b[91mnote doesn't have date\x1b[0m"),
        Some(v) => {
            line1_len = v.len();
            v
        },
    };
    let date = match DateTime::<Utc>::from_str(date_str) {
        Err(e) => panic!("\x1b[91mnote has invalid date: {e:?}\x1b[0m"),
        Ok(v) => v.to_utc(),
    };
    *note = note.split_at(line1_len + 1).1.to_string();
    date
}

pub fn set_note_date(note: String, date: DateTime<Utc>) -> String {
    let date_str = DateTime::<FixedOffset>::from(date).to_rfc3339();
    format!("{date_str}\n{note}")
}

pub fn get_note_title(note: &str) -> &str {
    let mut line = "";
    for l in note.lines() {
        if !l.trim().is_empty() {
            line = l;
            break;
        }
    }
    line
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
