use std::io::IsTerminal;
use crate::store::{get_counter, get_notes_path};
use std::fs::read_dir;

mod store;

struct Conf {
	action: ConfAction,
}

enum ConfAction {
	Create,
	List,
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
	}
	eprintln!();
}

fn exit_error(text: &str, code: i32) -> ! {
	eprintln!("{text}");
	pause();
	std::process::exit(code);
}

fn main() {

	let mut conf = Conf {
		action: ConfAction::Create,
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
					s => {
						exit_error(&format!("Invalid argument {s:?}"), -1);
					}
				}
			}
		}
	}

	store::recreate_data_folder();

	match conf.action {
		ConfAction::Create => {
			let t = chrono::Local::now();
			let t_fmt = t.format("%Y-%m-%d %H:%M:%S %Z");

			std::fs::write(
				store::get_edit_note_path(),
				format!("\n\n\n\n\
					# Adding new note.\n\
					# Lines starting with '#' are comments.\n\
					# Empty note aborts operation.\n\
					# time: {t_fmt}").as_bytes(),
			)
				.unwrap();

			std::process::Command::new("cmd")
				.arg("/c")
				.arg("vim")
				.arg(store::get_edit_note_path())
				.status()
				.unwrap();

			let note = std::fs::read_to_string(
				store::get_edit_note_path()
			)
				.unwrap();

			let mut note_lines = Vec::new();
			let mut is_empty_note = true;
			for line in note.lines() {
				let line = line.trim();
				if line.starts_with("#") {
					continue;
				}
				if is_empty_note && !line.is_empty() {
					is_empty_note = false;
				}
				note_lines.push(line);
			}

			if !is_empty_note {
				let note = note_lines.join("\n");

				let counter = get_counter();
				let new_note_name = format!("{counter:05}.md");
				let new_note_path = get_notes_path().join(new_note_name);

				match std::fs::write(new_note_path, note) {
					Err(e) => {
						exit_error(&format!("Failed to create new note!\nError: {e}"), -1);
					}
					Ok(()) => {}
				}
			}
		}
		ConfAction::List => {
			let mut d = read_dir(get_notes_path()).unwrap();
			for p in d {
				let p = p.unwrap();
				let p = p.path();
				let note = std::fs::read_to_string(&p)
					.unwrap();

				let filename = p.file_name().unwrap().to_str().unwrap();

				println!("Note {}", filename);
				let mut skip_first = true;
				let mut n_empty_lines = 0;
				let mut line_counter = 0;
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
							line_counter += 1;
							println!("\x1b[90m{line_counter:4}\x1b[33m|\x1b[0m");
							n_empty_lines -= 1;
						}
					}
					line_counter += 1;
					println!("\x1b[90m{line_counter:4}\x1b[33m|\x1b[0m {}", line);
				}
				println!();
			}

			pause();
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
