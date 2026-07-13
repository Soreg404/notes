use std::io::{Read, Write};
use std::path::Path;

pub fn get_counter_path() -> &'static Path {
	Path::new("data/metadata/COUNTER")
}
pub fn get_notes_path() -> &'static Path {
	Path::new("data/objects-store")
}

pub fn recreate_data_folder() {
	let p_data = Path::new("data");
	let p_metadata = Path::new("data/metadata");
    let p_drafts = Path::new("data/drafts");
	let p_objects_store = Path::new("data/objects-store");
	if !p_data.is_dir() {
		std::fs::create_dir(p_data).unwrap();
	}
	if !p_metadata.is_dir() {
		std::fs::create_dir(p_metadata).unwrap();
	}
    if !p_drafts.is_dir() {
		std::fs::create_dir(p_drafts).unwrap();
    }
	if !get_counter_path().is_file() {
		std::fs::write(get_counter_path(), b"0").unwrap();
	}
	if !p_objects_store.is_dir() {
		std::fs::create_dir(p_objects_store).unwrap();
	}
}

pub fn get_counter() -> usize {
    use std::io::Seek;
    let mut counter_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(Path::new("data/metadata/COUNTER"))
        .unwrap();
    counter_file.lock()
        .unwrap();
    let mut contents = String::new();
    counter_file.read_to_string(&mut contents)
        .unwrap();
    let contents = contents.trim();
    let counter = contents.parse::<usize>()
        .unwrap();
    counter_file.rewind()
        .unwrap();
    counter_file.write_all(format!("{}\n", counter + 1).as_bytes())
        .unwrap();
    counter
}
