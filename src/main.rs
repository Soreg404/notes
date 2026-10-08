#![deny(warnings)]

use std::{
    io::Write,
    fs::read_dir,
};
use chrono::Utc;
use chrono::DateTime;
use crate::store::{get_notes_path};

mod config;
mod store;
mod browser_open;
mod exit_process;
use exit_process::proc_exit;

mod helpers;
mod create_note;
mod edit_note;

fn main() {
    let conf = config::get_app_config();

    store::recreate_data_folder();

    match conf.action {
        config::Action::Create => {
            create_note::create_note();
        }
        config::Action::Edit(n) => {
            edit_note::edit_note(n);
        }
        action => {
            let mut pager_proc = create_pager_proc();
            let mut pager_stdin = pager_proc.stdin.take().unwrap();

            match action {
                config::Action::Create => unreachable!(),
                config::Action::Edit(_) => unreachable!(),
                config::Action::List { slim } => {
                    let d = read_dir(get_notes_path()).unwrap();

                    let mut note_paths = Vec::new();
                    for p in d {
                        note_paths.push(p.unwrap());
                    }

                    for p in note_paths.iter().rev() {
                        let p = p.path();
                        let note = std::fs::read_to_string(&p)
                            .unwrap();

                        let filename = p.file_name().unwrap().to_str().unwrap();
                        let filedate = {
                            let secs = std::fs::metadata(&p).unwrap().modified().unwrap()
                                .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                            chrono::DateTime::<Utc>::from_timestamp_secs(secs as i64).unwrap()
                        };

                        let date_relative_str = date_relative(&filedate);

                        if slim {
                            let title = get_note_title(&note);
                            let date_fmt = filedate.format("%Y-%m-%d");
                            let num_days = -(Utc::now() - filedate).num_days();
                            _ = writeln!(pager_stdin, "{filename} {date_fmt} {num_days:4} \x1b[33m|\x1b[0m {title}");
                        } else {
                            _ = writeln!(pager_stdin, "{filename} {} ({date_relative_str})", filedate.format("%Y-%m-%d %H:%M:%S"));
                            let mut skip_first = true;
                            let mut n_empty_lines = 0;
                            let mut line_counter = 1;
                            for line in note.lines() {
                                if line.trim().is_empty() {
                                    if skip_first {
                                        line_counter += 1;
                                        continue;
                                    }

                                    n_empty_lines += 1;
                                    continue;
                                } else {
                                    skip_first = false;
                                    while n_empty_lines > 0 {
                                        _ = writeln!(pager_stdin, "  {line_counter:4} \x1b[33m|\x1b[0m");
                                        line_counter += 1;
                                        n_empty_lines -= 1;
                                    }
                                }
                                _ = writeln!(pager_stdin, "  {line_counter:4} \x1b[33m|\x1b[0m {line}");
                                line_counter += 1;
                            }
                            _ = writeln!(pager_stdin);
                        }
                    }
                }
                config::Action::Show { n, browser } => {
                    let note_name = format!("{n:08}.md");
                    let filename = get_notes_path().join(&note_name);         

                    if !std::fs::exists(&filename).unwrap() {
                        proc_exit(Err(format!("note {n} does not exist")));
                    }

                    let mut proceed_show = true;
                    if browser {
                        match browser_open::open(&filename) {
                            Ok(()) => { proceed_show = false } 
                            _ => {
                                _ = writeln!(pager_stdin,
                                    "couldn't open note {filename:?} in browser");
                            }
                        };
                    }

                    if proceed_show {
                        let filedate = {
                            let secs = std::fs::metadata(&filename).unwrap().modified().unwrap()
                                .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                            chrono::DateTime::<Utc>::from_timestamp_secs(secs as i64).unwrap()
                        };
                        let date_relative_str = date_relative(&filedate);

                        let note = match std::fs::read_to_string(filename){
                            Err(_) => {
                                proc_exit(Err(format!("failed to read note {n}")));
                            }
                            Ok(v) => v
                        };

                        _ = writeln!(
                            pager_stdin,
                            "{note_name} {} ({date_relative_str})",
                            filedate.format("%Y-%m-%d %H:%M:%S")
                        );
                        let mut skip_first = true;
                        let mut n_empty_lines = 0;
                        let mut line_counter = 1;
                        for line in note.lines() {
                            if line.trim().is_empty() {
                                if skip_first {
                                    line_counter += 1;
                                    continue;
                                }

                                n_empty_lines += 1;
                                continue;
                            } else {
                                skip_first = false;
                                while n_empty_lines > 0 {
                                    _ = writeln!(pager_stdin, "  {line_counter:4} \x1b[33m|\x1b[0m");
                                    line_counter += 1;
                                    n_empty_lines -= 1;
                                }
                            }
                            _ = writeln!(pager_stdin, "  {line_counter:4} \x1b[33m|\x1b[0m {line}");
                            line_counter += 1;
                        }
                        _ = writeln!(pager_stdin);
                    }
                }
            }

            drop(pager_stdin);
            pager_proc.wait().unwrap();
        }
    }
}

fn get_note_title(note: &str) -> &str {
    for line in note.lines() {
        if !line.trim().is_empty() {
            return line;
        }
    }
    panic!("empty note is not allowed")
}

fn date_relative(datetime: &DateTime<Utc>) -> String {
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


fn create_pager_proc() -> std::process::Child {
    std::process::Command::new("cmd")
        .arg("/c")
        .arg("less")
        .arg("-RFSI")
        .stdout(std::process::Stdio::inherit())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap()
}
