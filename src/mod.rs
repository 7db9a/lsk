pub mod list;
pub mod terminal;

use std::path::{Path, PathBuf};
use std::fs::metadata;
use list::List;
use fixture::{command_assistors, Fixture};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LsKey {
    list: List,
    all: bool,
}

impl LsKey {
    pub fn new<P: AsRef<Path>>(path: P, all: bool) -> Self {
            let list = if all {
               list::List::new(path)
                   .list_include_hidden()
                   .unwrap()
            } else {
               list::List::new(path)
                   .list_skip_hidden()
                   .unwrap()
            };

            LsKey {
                list,
                all
            }
    }

    pub fn update(mut self, list: List) -> Self {
            let list = if self.all {
                   list
                   .list_include_hidden()
                   .unwrap()
            } else {
                   list
                   .list_skip_hidden()
                   .unwrap()
            };
            
            self.list = list;
            self.clone()
    }

    pub fn run_list_read(self) {
            let list = self.list.clone();
            let entries: Vec<PathBuf> = list::order_and_sort_list(list.clone());

            let entries_keyed: Vec<String> = list::key_entries(entries);
            //let res = terminal::input_n_display::grid(entries_keyed);
            let res = terminal::input_n_display::grid(entries_keyed);
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
            self.run_cmd(list);
    }

    fn run_cmd(mut self, list: List) {
        let input = terminal::input_n_display::read();
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
                            let mut path_cache = command_assistors::PathCache::new(
                                self.list.relative_parent_dir_path.as_path()
                            );
                            path_cache.switch();
                            terminal::shell::spawn(cmd, args);
                            path_cache.switch_back();
                            self.run_list_read();
                        } else {
                            let as_read = input.as_read.as_str();
                            println!("\n\nInput: {}", as_read);
                            let work = "w";
                            let quite = "q";
                            let fzf = "f";
                            match as_read {
                                "w" => {
                                     // Cd the parent shell into the directory viewed by ls-key.
                                     let path = self.list.relative_parent_dir_path;
                                     let path = path.to_str().unwrap();
                                     let cmd = format!(r#""$(printf 'cd {} \n ')""#, path).to_string();
                                     terminal::parent_shell::type_text(cmd, 0);
                                },
                                "q" => (),
                                _ => {
                                    let mut path_cache = command_assistors::PathCache::new(
                                        self.list.relative_parent_dir_path.as_path()
                                    );
                                    path_cache.switch();
                                    terminal::shell::cmd(as_read.to_string());
                                    path_cache.switch_back();
                                    self.run_list_read();
                                }
                            }
                        }
                    } else if input.is_key == Some(true) {
                        let key: usize = input.cmd.unwrap().parse().unwrap();
                        match key {
                            0 => {
                                 self.list.relative_parent_dir_path.pop();
                                 let file_pathbuf = self.list.relative_parent_dir_path.clone();
                                 self.list.relative_parent_dir_path.pop();
                                 let list = self.list.clone().update(file_pathbuf);
                                 self = self.update(list);
                                 self.run_list_read();
                            },
                            _ => {
                                  let file_pathbuf = list.get_file_by_key(key).unwrap();
                                  if metadata(file_pathbuf.clone()).unwrap().is_dir() {
                                      let file_path =
                                          file_pathbuf
                                          .to_str().unwrap()
                                          .to_string();

                                      let list = self.list.clone().update(file_pathbuf);
                                      self = self.update(list);
                                      self.run_list_read();
                                  } else {
                                      let file_path =
                                          file_pathbuf
                                          .to_str().unwrap()
                                          .to_string();
                                      terminal::shell::spawn("vim".to_string(), vec![file_path]);
                                      self.run_list_read();
                                  }
                            }
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
    use super::{Input, LsKey};


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
        let spawn = super::terminal::parent_shell::type_text_spawn(text_vec, 200);
        //let spawn_quite = super::terminal::parent_shell::type_text_spawn(r#""$(printf ':q \n ')""#, 700);
        let ls_key = super::LsKey::new(path, true);
        ls_key.run_list_read();
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
        let spawn = super::terminal::parent_shell::type_text_spawn(text_vec, 200);
        //let spawn_quite = super::terminal::parent_shell::type_text_spawn(r#""$(printf ':q \n ')""#, 700);
        let ls_key = super::LsKey::new(path, true);
        ls_key.run_list_read();
        spawn.join();
        //spawn_quite.join();
    }
}
