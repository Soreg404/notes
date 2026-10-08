use std::{
    io::{Read, Write},
    fs::File,
    path::{
        Path,
        PathBuf,
    },
};

pub fn notes_path() -> &'static Path {
    &Path::new("data/objects-store")
}

pub fn recreate_data_folder() {
	let p_data = Path::new("data");
	if !p_data.is_dir() {
		std::fs::create_dir(p_data).unwrap();
	}
    
	let p_metadata = Path::new("data/metadata");
	if !p_metadata.is_dir() {
		std::fs::create_dir(p_metadata).unwrap();
	}
    let p_meta_version = Metadata::Version.as_path_buf();
	if !p_meta_version.is_file() {
		std::fs::write(p_meta_version, b"0").unwrap();
	}
    let p_meta_counter = Metadata::Counter.as_path_buf();
	if !p_meta_counter.is_file() {
		std::fs::write(p_meta_counter, b"0").unwrap();
	}

    let p_drafts = Path::new("data/drafts");
    if !p_drafts.is_dir() {
		std::fs::create_dir(p_drafts).unwrap();
    }

	let p_objects_store = Path::new("data/objects-store");
	if !p_objects_store.is_dir() {
		std::fs::create_dir(p_objects_store).unwrap();
	}
}

#[derive(Copy, Clone)]
pub enum Metadata {
    Version,
    Counter,
}
impl Metadata {
    fn as_path_buf(&self) -> PathBuf {
        Path::new("data/metadata")
            .join(match self {
                Metadata::Version => "VERSION",
                Metadata::Counter => "COUNTER",
            })
    }
}

pub fn get_version() -> usize {
    let mut num = 0usize;
    meta_file_lock(
        Metadata::Version,
        |s| {
            num = s.parse::<usize>().unwrap();
        }
    );
    num
}

pub fn set_version(ver: usize) {
    println!("\x1b[93mset new version: {ver}\x1b[0m");
    std::fs::write(
        Metadata::Version.as_path_buf().as_path(),
        ver.to_string(),
    ).unwrap();
}

pub fn get_counter_string() -> String {
    let mut num = 0usize;
    meta_file_lock(
        Metadata::Counter,
        |s| {
            num = s.parse::<usize>().unwrap();
            *s = (num + 1).to_string();
        }
    );
    format!("{num:08}")
}


fn meta_file_lock(which: Metadata, func: impl FnOnce(&mut String)) {
    let p = &which.as_path_buf();
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&p)
        .unwrap();

    file.lock()
        .unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .unwrap();
    let mut contents = contents.trim().to_string();

    func(&mut contents);

    <File as std::io::Seek>::rewind(&mut file)
        .unwrap();
    file.write_all(contents.as_bytes())
        .unwrap();
}
