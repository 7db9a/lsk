pub mod list;
pub mod terminal;

use std::path::{Path, PathBuf};
use std::fs::metadata;
use list::List;

pub fn run_list_read<P: AsRef<Path>>(path: P, all: bool) {
        let list = if all {
           list::List::new(path)
               .list_include_hidden()
               .unwrap()
        } else {
           list::List::new(path)
               .list_skip_hidden()
               .unwrap()
        };

        let entries: Vec<PathBuf> = list::order_and_sort_list(list.clone());

        let entries_keyed: Vec<String> = list::key_entries(entries);
        //let res = terminal::terminal_n_grid::grid(entries_keyed);
        let res = terminal::terminal_n_grid::_grid(entries_keyed);
        if let Some(r) = res {
            let grid = r.0;
            let width = r.1;
            let display = grid.fit_into_width(width);
            if let Some(d) = display {
                 println!("{}", d);
            } else {
                list::print_list_with_keys(list.clone());
            }
        } else {
            list::print_list_with_keys(list.clone());
        }
        run_cmd(list, all);
}

fn run_cmd(list: List, all: bool) {
    let input = terminal::termion::read();
    match input {
        Ok(t) =>  {
            if let Some(i) = t {
                let input = Input::new();
                let input = input.parse(i);
                if input.is_key == Some(false) {
                    let args = input.args;
                    if let Some(a) = args {
                        let args = a;
                        // Unwrap is safe because is_key is not None and there are args.
                        let cmd = input.cmd.unwrap();
                        terminal::shell::spawn(cmd, args);
                    } else {
                        let as_read = input.as_read;
                        terminal::shell::cmd(as_read);
                    }
                } else if input.is_key == Some(true) {
                    let key: usize = input.cmd.unwrap().parse().unwrap();
                    let file_path = list.get_file_by_key(key);
                    if metadata(file_path.clone().unwrap()).unwrap().is_dir() {
                        let file_path =
                            file_path.unwrap()
                            .to_str().unwrap()
                            .to_string();

                        run_list_read(file_path, all);
                    } else {
                        let file_path =
                            file_path.unwrap()
                            .to_str().unwrap()
                            .to_string();
                        terminal::shell::spawn("vim".to_string(), vec![file_path]);
                    }
                } else {
                }

                ()
            } else {
                ()
            }
        },
        Err(_) => ()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
struct Input {
    cmd: Option<String>,
    args: Option<Vec<String>>,
    is_key: Option<bool>,
    as_read: String
}

impl Input {
    fn new() -> Self {
        let input: Input = Default::default();

        input
    }

    fn defang_args(&self, args: Vec<String>) -> Option<Vec<String>> {
        let count = args.clone().iter().count();
        let empty_item = args.clone().iter().any(|x| *x == "".to_string());
        let is_valid = if empty_item && count <= 1 {
            false
        } else {
            true
        };

        if is_valid && count != 0 {
            Some(args)
        } else {
            None
        }
    }

    fn parse(mut self, input: String) -> Self {
        let (cmd, args) = self.parse_cmd(input.clone());
        let args_count = args.clone().iter().count();
        let is_key = if args == None && cmd != None {
            let key: Result<(usize), std::num::ParseIntError> = cmd.clone().unwrap().parse();
            match key {
                Ok(_) => Some(true),
                Err(_) => Some(false)
            }
        } else if cmd == None {
            None
        } else {
            Some(false)
        };

        self.cmd = cmd;
        self.args = args;
        self.is_key = is_key;
        self.as_read = input;

        self
    }

    fn parse_cmd(&self, input: String) -> (Option<String>, Option<Vec<String>>) {
        let mut input: Vec<String> = input.clone().split(" ").map(|s| s.to_string()).collect();
        let cmd = input.remove(0);

        if cmd == "".to_string() {
            (None, None)
        } else {
            let args = self.defang_args(input);
            (Some(cmd), args)
        }
     }

     fn is_key(&self, input: &Vec<String>) -> bool {
        if input.iter().count() == 1 {
            let key: Result<(usize), std::num::ParseIntError> = input.iter().next().unwrap().parse();
            match key {
                Ok(_) => true,
                Err(_) => false
            }
        } else {
            false
        }
     }
}

#[cfg(test)]
mod tests {
    use std::fs::metadata;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::env;
    use fixture::Fixture;
    use super::Input;


    #[test]
    #[ignore]//host
    fn parse() {
        let input = Input::new();
        let input = input.parse("vim Cargo.toml".to_string());

        assert_eq!(
           Some(false),
           input.is_key
        );

        assert_eq!(
           Some("vim".to_string()),
           input.cmd
        );

        assert_eq!(
           Some(vec!["Cargo.toml".to_string()]),
           input.args
        );
    }

    #[test]
    #[ignore]//host
    fn parse_long() {
        let input = Input::new();
        let input = input.parse("git clone https://github.com/7db9a/ls-key --depth 1".to_string());

        assert_eq!(
           Some(false),
           input.is_key
        );

        //vec!["clone".to_string(), "https://github.com/7db9a/ls-key".to_string(), "--depth".to_string(), "1".to_string()]

        assert_eq!(
           Some("git".to_string()),
           input.cmd
        );

        assert_eq!(
           Some(vec![
                "clone".to_string(),
                "https://github.com/7db9a/ls-key".to_string(),
                "--depth".to_string(),
                "1".to_string()
           ]),
           input.args
        );
    }

    #[test]
    #[ignore]//host
    fn parse_single_cmd() {
        let input = Input::new();
        let input = input.parse("vim".to_string());

        assert_eq!(
           Some(false),
           input.is_key
        );

        assert_eq!(
           Some("vim".to_string()),
           input.cmd
        );

        assert_eq!(
           None,
           input.args
        );
    }

    #[test]
    #[ignore]//host
    fn parse_key() {
        let input = Input::new();
        let input = input.parse("33".to_string());

        assert_eq!(
           Some(true),
           input.is_key
        );

        assert_eq!(
           Some("33".to_string()),
           input.cmd
        );

        assert_eq!(
           None,
           input.args
        );
    }

    #[test]
    #[ignore]//host
    fn parse_bad() {
        let input = Input::new();
        let input = input.parse(" vim Cargo.toml".to_string());

        assert_eq!(
           None,
           input.is_key
        );

        assert_eq!(
           None,
           input.cmd
        );

        assert_eq!(
           None,
           input.args
        );
    }

    #[test]
    #[ignore]//host
    fn parse_cmd() {
        let input = Input::new();
        let (cmd, args) = input.parse_cmd("vim Cargo.toml".to_string());

        assert_eq!(
            cmd,
            Some("vim".to_string())
        );

        assert_eq!(
            args,
            Some(vec!["Cargo.toml".to_string()])
        )
    }

    //#[test]
    fn shell_spawn_vim() {
        super::terminal::shell::spawn("vim".to_string(), vec!["-c".to_string(), "vsplit README.md".to_string(), "dev.sh".to_string()]);
    }

    //#[test]
    #[ignore]
    fn shell_pipe_cmd() {
        super::terminal::shell::cmd(r#"du -ah . | sort -hr | head -n 10"#.to_string());
    }

    //#[test]
    #[ignore]
    fn shell_cat_cmd() {
        super::terminal::shell::cmd("cat Cargo.toml".to_string());
    }

    //#[test]
    #[ignore]
    fn shell_cat() {
        super::terminal::shell::spawn("cat".to_string(), vec!["Cargo.toml".to_string()]);
    }

    #[test]
    #[ignore]//host
    fn takes_input_run_list_read() {
        let path = env::current_dir().unwrap();
        println!("");
        let text_vec = vec![
             r#""$(printf '2 \n ')""#.to_string(),
             r#""$(printf ':q \n ')""#.to_string(),
        ];
        let spawn = super::terminal::xdotool::type_text_spawn(text_vec, 200);
        //let spawn_quite = super::terminal::xdotool::type_text_spawn(r#""$(printf ':q \n ')""#, 700);
        super::run_list_read(path, true);
        spawn.join();
        //spawn_quite.join();
    }

    #[test]
    #[ignore]//host
    fn takes_input_run_list_all_read() {
        let path = env::current_dir().unwrap();
        println!("");
        let text_vec = vec![
             r#""$(printf '7 \n ')""#.to_string(),
             r#""$(printf ':q \n ')""#.to_string(),
        ];
        let spawn = super::terminal::xdotool::type_text_spawn(text_vec, 200);
        //let spawn_quite = super::terminal::xdotool::type_text_spawn(r#""$(printf ':q \n ')""#, 700);
        super::run_list_read(path, true);
        spawn.join();
        //spawn_quite.join();
    }
}
