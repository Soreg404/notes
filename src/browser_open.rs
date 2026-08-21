use std::path::Path;

pub fn open(filepath: &Path) -> Result<(), ()> {
    let cmd_raw = match get_browser_cmd_string() {
        Some(s) => s,
        None => return Err(())
    };

    let filepath = std::path::absolute(filepath).unwrap();
    let url = format!("\"file:///{}\"", filepath.to_string_lossy().replace("\\", "/"));
    let cmd_composed = cmd_raw.replace("%1", &url);

    let cmd_args = split_cmd(&cmd_composed);

    match std::process::Command::new(&cmd_args[0])
        .args(&cmd_args[1..])
        .spawn() {
            Ok(_) => return Ok(()),
            Err(_) => return Err(())
        }
}

fn split_cmd(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    args.push(String::new());
    let mut inq = false;
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == ' ' && !inq {
            while let Some(c) = it.peek() {
                if *c != ' ' {
                    break;
                }
                _ = it.next();
            }
            if it.peek().is_none() {
                break;
            }
            args.push(String::new());
            continue;
        }
        if c == '"' {
            inq = !inq;
            continue;
        }
        args.last_mut().unwrap().push(c);
    }
    args
}

const ASSOCF_IS_PROTOCOL: u32 = 0x00001000;
const ASSOCSTR_COMMAND: i32 = 1;

#[link(name="shlwapi")]
unsafe extern "system" {
    fn AssocQueryStringW(
        flags: u32,
        a_str: i32,
        pszAssoc: *const u16,
        pszExtra: *const u16,
        pszOut: *mut u16,
        pcchOut: *mut u32
    ) -> i32;
}

fn get_browser_cmd_string() -> Option<String> {
    unsafe {
        let mut cmd_buf = [0u16; 0x800];
        let mut buf_len: u32 = 0x800;
        let r = AssocQueryStringW(
            ASSOCF_IS_PROTOCOL,
            ASSOCSTR_COMMAND,
            ar16_from([b'h', b't', b't', b'p', 0]).as_ptr(),
            std::ptr::null(),
            cmd_buf.as_mut_ptr(),
            &mut buf_len
        );
        if r != 0 {
            return None;
        }

        let cmd_buf = &cmd_buf[..buf_len as usize - 1];
        Some(char::decode_utf16(cmd_buf.iter().copied())
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect::<String>())
    }
}

fn ar16_from<const S: usize>(u8a: [u8; S]) -> [u16; S] {
    let mut u16a = [0u16; S];
    for i in 0..S {
        u16a[i] = u8a[i] as u16;
    }
    u16a
}
