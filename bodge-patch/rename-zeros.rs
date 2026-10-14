fn main() {
    for p in std::fs::read_dir("objects-store").unwrap() {
        let p = p.unwrap().path();
        println!("doing path {p:?}");
        let stem = p.file_stem().unwrap();
        let stem = stem.to_string_lossy();
        println!("stem: {stem:?}");
        let n = stem.parse::<usize>().unwrap();
        println!("n={n:?}");
        if stem.len() != 8 {
            let parent = p.parent().unwrap();
            let p2 = parent.join(format!("{n:08}.md"));
            println!("rename to {p2:?}");
            std::fs::rename(p, p2).unwrap();
        } else {
            println!("skip");
        }
    }
}
