#![deny(warnings)]

use std::{
    io::Write,
    fs::read_dir,
};
use crate::store::{get_notes_path};

use chrono::{
    Utc,
};

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
            edit_note::edit_note(&format!("{:08}", n));
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
                        let note = helpers::patch_create_note_date_from_ftime(note, &p);
                        let filename = p.file_name().unwrap().to_str().unwrap();


                        let date = helpers::get_note_date(&note);
                        let date_relative_str = helpers::date_relative_str(date);

                        if slim {
                            let title = helpers::get_note_title(&note);
                            let date_fmt = date.format("%Y-%m-%d");
                            let num_days = -(Utc::now() - date).num_days();
                            _ = writeln!(pager_stdin, "{filename} {date_fmt} {num_days:4} \x1b[33m|\x1b[0m {title}");
                        } else {
                            _ = writeln!(pager_stdin, "{filename} {} ({date_relative_str})", date.format("%Y-%m-%d %H:%M:%S"));
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
                        let note = match std::fs::read_to_string(&filename){
                            Err(ioerr) => {
                                proc_exit(Err(format!("failed to read note {n}: {ioerr:?}")));
                            }
                            Ok(v) => v
                        };

                        let date = helpers::get_note_date(&note);
                        let note = helpers::patch_create_note_date_from_ftime(
                            note, &filename);

                        let date_relative_str = helpers::date_relative_str(date);

                        _ = writeln!(
                            pager_stdin,
                            "{note_name} {} ({date_relative_str})",
                            date.format("%Y-%m-%d %H:%M:%S")
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
