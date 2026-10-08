use crate::exit_process::proc_exit;

pub struct AppConfig {
    pub action: Action,
}

pub enum Action {
    Create,
    List {
        slim: bool
    },
    Show {
        n: usize,
        browser: bool,
    },
    Edit(usize),
}

pub fn get_app_config() -> AppConfig {
    let mut conf = AppConfig {
        action: Action::Create
    };

    let mut action_arg_set = false;
    let mut check_action_arg_set = |foul: &str| {
        if action_arg_set {
            proc_exit(Err(format!("{foul}: only one action argument allowed")));
        }
        action_arg_set = true;
    };

    for arg in std::env::args().skip(1) {
        let arg = arg.as_str();
        match arg {
            "--create" | "-c" => {
                check_action_arg_set(arg);
                conf.action = Action::Create;
            }
            "--list" | "-l" => {
                check_action_arg_set(arg);
                conf.action = Action::List {
                    slim: false
                };
            }
            "-s" | "--slim" => {
                match &mut conf.action {
                    Action::List { slim } => *slim = true,
                    _ => proc_exit(Err(format!("{arg}: --list must be set first"))),
                }
            }
            "-b" | "--browser" => {
                match &mut conf.action {
                    Action::Show { browser, .. } => *browser = true,
                    _ => proc_exit(Err(format!("{arg}: --show=<n> must be set first"))),
                }
            }
            s => {
                if s.starts_with("-n=") || s.starts_with("--show=") {
                    let eqpos = s.chars().position(|c| c == '=')
                        .unwrap();
                    let arg = s.chars().take(eqpos).collect::<String>();
                    let num_s = s.chars().skip(eqpos + 1).collect::<String>();
                    let n = match num_s.parse::<usize>() {
                        Ok(v) => v,
                        Err(_) => {
                            proc_exit(Err(format!("Invalid value for arg {arg}")));
                        }
                    };
                    conf.action = Action::Show {
                        n,
                        browser: false,
                    };
                }
                else if s.starts_with("-e=") || s.starts_with("--edit=") {
                    let eqpos = s.chars().position(|c| c == '=')
                        .unwrap();
                    let arg = s.chars().take(eqpos).collect::<String>();
                    let num_s = s.chars().skip(eqpos + 1).collect::<String>();
                    let n = match num_s.parse::<usize>() {
                        Ok(v) => v,
                        Err(_) => {
                            proc_exit(Err(format!("Invalid value for arg {arg}")));
                        }
                    };
                    conf.action = Action::Edit(n);
                }
                else {
                    proc_exit(Err(format!("Invalid argument {s:?}")));
                }
            }
        }
    }

    conf
}
