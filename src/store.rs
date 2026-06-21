
use std::path::Path;

pub fn get_counter_path() -> &'static Path {
	Path::new("data/metadata/COUNTER")
}
pub fn get_edit_note_path() -> &'static Path {
	Path::new("data/metadata/CURRENT_NOTE")
}
pub fn get_notes_path() -> &'static Path {
	Path::new("data/objects-store")
}

pub fn recreate_data_folder() {
	let p_data = Path::new("data");
	let p_metadata = Path::new("data/metadata");
	let p_objects_store = Path::new("data/objects-store");
	if !p_data.is_dir() {
		std::fs::create_dir(p_data).unwrap();
	}
	if !p_metadata.is_dir() {
		std::fs::create_dir(p_metadata).unwrap();
	}
	if !get_counter_path().is_file() {
		std::fs::write(get_counter_path(), b"0").unwrap();
	}
	if !get_edit_note_path().is_file() {
		std::fs::write(get_edit_note_path(), b"").unwrap();
	}
	if !p_objects_store.is_dir() {
		std::fs::create_dir(p_objects_store).unwrap();
	}
}

fn get_counter_() -> usize {
	std::fs::read_to_string(get_counter_path())
		.unwrap()
		.trim_ascii()
		.parse::<usize>()
		.unwrap()
}

fn set_counter(new_counter: usize) {
	std::fs::write(
		get_counter_path(),
		new_counter
			.to_string()
			.as_bytes(),
	)
		.unwrap()
}

pub fn get_counter() -> usize {
	let tmp = get_counter_();
	set_counter(tmp + 1);
	tmp
}
