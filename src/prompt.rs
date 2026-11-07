pub fn prompt_yn() -> bool {
    loop {
        println!("continue? y/N");
        let mut s = String::new();
        std::io::stdin().read_line(&mut s).unwrap();
        match s.trim() {
            "y" | "Y" => {
                println!("continue");
                return true;
            }
            "n" | "N" | "" => {
                println!("cancel");
                return false;
            }
            _ => continue
        }
    }
}
