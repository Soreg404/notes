use std::io::IsTerminal;

unsafe extern "system" {
    fn _getch() -> i32;
    fn _kbhit() -> i32;
}

fn pause() {
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

pub fn proc_exit(how: Result<Option<String>, String>) -> ! {
    let exitcode = match how {
        Ok(None) => 0,
        Ok(Some(s)) => {
            println!("exit ok: {s}");
            0
        },
        Err(s) => {
            eprintln!("exit error: {s}");
            -2
        }
    };
    if std::io::stdout().is_terminal() {
        pause();
    }
    std::process::exit(exitcode);
}
