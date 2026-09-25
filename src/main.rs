#![deny(unused_must_use)]

use std::io::IsTerminal;
use crate::store::{get_notes_path};
use std::fs::read_dir;
use chrono::Utc;
use chrono::DateTime;
use std::io::Write;
use std::process::exit;
use std::path::{Path, PathBuf};

mod store;
mod browser_open;

// todo: redo conf args
struct Conf {
    action: ConfAction,
    slim: bool,
    browser_open: bool,
}

#[derive(PartialEq, Eq)]
enum ConfAction {
    Create,
    List,
    Show(usize)
}

unsafe extern "system" {
    fn _getch() -> i32;
    fn _kbhit() -> i32;
}

fn pause() {
    if !std::io::stdout().is_terminal() {
        return;
    }

    unsafe {
        while _kbhit() != 0 {
            _ = _getch();
        }
        eprint!("\nPress any key to continue...");
        _ = _getch();
        while _kbhit() != 0 {
            _ = _getch();
        }
    }
    eprintln!();
}

fn exit_error(text: &str, code: i32) -> ! {
    eprintln!("{text}");
    pause();
    exit(code);
}

fn main() {

    let mut conf = Conf {
        action: ConfAction::Create,
        slim: false,
        browser_open: false,
    };

    let mut args_it = std::env::args().skip(1);
    loop {
        match args_it.next() {
            None => break,
            Some(s) => {
                match s.as_str() {
                    "--create" | "-c" => {
                        conf.action = ConfAction::Create;
                    }
                    "--list" | "-l" => {
                        conf.action = ConfAction::List;
                    }
                    "-s" | "--slim" => {
                        conf.slim = true;
                    }
                    "-b" | "--browser" => {
                        conf.browser_open = true;
                    }
                    s => {
                        if s.starts_with("-n=") {
                            let num_s = &s[3..];
                            let num = match num_s.parse::<usize>() {
                                Ok(v) => v,
                                Err(_) => {
                                    exit_error(&format!("Invalid value for arg -n"), -1);
                                }
                            };
                            conf.action = ConfAction::Show(num);
                        } else {
                            exit_error(&format!("Invalid argument {s:?}"), -1);
                        }
                    }
                }
            }
        }
    }

    store::recreate_data_folder();


    if conf.action == ConfAction::Create {
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
        let note = std::fs::read_to_string(
            note_draft_path
        )
            .unwrap();

        let note = {
            let mut tmp = Vec::<u8>::new();
            let note: &[u8] = note.as_bytes();
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
        };

        let mut note_lines = Vec::new();
        let mut is_empty_note = true;
        for line in note.lines() {
            if is_empty_note && !line.trim().is_empty() {
                is_empty_note = false;
            }
            note_lines.push(line.trim_end());
        }

        if is_empty_note {
            eprintln!("Empty note, aborting.");
            std::fs::remove_file(note_draft_path)
                .unwrap();
            exit(10);
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
    } else {
        let mut pager_proc = create_pager_proc();
        let mut pager_stdin = pager_proc.stdin.take().unwrap();

        match conf.action {
            ConfAction::Create => unreachable!(),
            ConfAction::List => {
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

                    if conf.slim {
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
            ConfAction::Show(n) => {
                let note_name = format!("{n:05}.md");
                let filename = get_notes_path().join(&note_name);         

                let mut proceed_show = true;
                if conf.browser_open {
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
                            exit_error(&format!("failed to read note {n}"), -2);
                        }
                        Ok(v) => v
                    };

                    _ = writeln!(pager_stdin, "{note_name} {} ({date_relative_str})", filedate.format("%Y-%m-%d %H:%M:%S"));
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
        pager_proc
            .wait()
            .unwrap();
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
        format!("{} month(s) ago", time_diff.num_days() / 30)
    } else {
        format!("{} year(s) ago", time_diff.num_days() / 365)
    }
}


fn create_pager_proc() -> std::process::Child {
    std::process::Command::new("cmd")
        .arg("/c")
        .arg("less")
        .arg("-RFS")
        .stdout(std::process::Stdio::inherit())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap()
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
