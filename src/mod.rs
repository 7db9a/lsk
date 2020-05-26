pub mod list;
pub mod terminal;

use std::path::{Path, PathBuf};
use std::fs::metadata;
use list::List;
use fixture::{command_assistors, Fixture};
use termion::input::TermRead;
use termion::event::Key;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::terminal_size;
use term_grid::{/*Grid,*/ GridOptions, Direction, /*Display,*/ Filling, Cell};
use std::io::{Read, Write, stdout, stdin, Stdout, StdoutLock};
use std::convert::TryFrom;
use termion::async_stdin;
use termion::screen::AlternateScreen;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LsKey {
    list: List,
    all: bool,
    display: Option<(PathBuf, String)>
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
                all,
                display: None,
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

   pub fn run_list_read(mut self) {
            let list = self.list.clone();
            let entries: Vec<PathBuf> = list::order_and_sort_list(list.clone());

            let entries_keyed: Vec<String> = list::key_entries(entries);
            //let res = terminal::input_n_display::grid(entries_keyed);
            let res = terminal::input_n_display::grid(entries_keyed);
            let mut show = "".to_string();
            if let Some(r) = res {
                let grid = r.0;
                let width = r.1;
                let display = grid.fit_into_width(width);
                if let Some(d) = display {
                     //println!("\n\n{}", d);
                     self.display = Some((self.list.relative_parent_dir_path.clone(), d.to_string()));
                } else {
                    //println!("\n\n");
                    //list::print_list_with_keys(list.clone());
                }
            } else {
                //println!("\n\n");
                //list::print_list_with_keys(list.clone());
            }
            self.run_cmd(list);
    }

    fn return_file_by_key_mode(mut self, list: List, input: Input) {
        let get_file = |key_string: String| {
             let key: usize = key_string.parse().unwrap();
             self.list.get_file_by_key(key).unwrap()
        };

        let mut n = 0;
        let mut format_cmd = |key: PathBuf| {
                    n +=1;
                   format!(r#"{}={}"#, n, get_file(key.to_str().unwrap().to_string()).to_str().unwrap().to_string())
        };

        if let Some (r) = input.args {
            let files_vec: Vec<&String> = vec![];
            let output_vec: Vec<std::process::Output> =
                r.iter()
                    .map(|mut key|
                         format_cmd(PathBuf::from(key))
                    ).map(|mut statement|
                        format!(r#""$(printf '{} \n ')""#, statement)
                    ).map(|mut cmd|
                        terminal::parent_shell::type_text(
                            cmd,
                            //format!(r#""$(printf '1={} && 2={} \n ')""#, "README", "LICENSE"),
                            0
                        )
                    ).collect();
        } else {
            ()
        }
    }

    fn key_mode(mut self, list: List, input: Input) {
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
    }

    fn cmd_mode(mut self, list: List, input: Input) {
         let args = input.args;
         if let Some(a) = args {
             let args = a;
             // Unwrap is safe because is_key is not None and there are args.
             let cmd = input.cmd.unwrap();
             let mut path_cache = command_assistors::PathCache::new(
                 self.list.relative_parent_dir_path.as_path()
             );
             path_cache.switch();
             match cmd.as_str() {
                 "fzf" => {
                     //Split cmd ('fzf')
                     let split: Vec<&str> = input.as_read.split("fzf").collect();
                     let cmd = split.iter().last().unwrap();
                     let cmd = format!(r#"fzf {}"#, cmd);
                     let output = terminal::shell::cmd(cmd.clone());
                     let file_path = output.unwrap();
                     terminal::shell::spawn("vim".to_string(), vec![file_path]);
                 }
                 _ => {
                      terminal::shell::spawn(cmd.to_string(), args);
                 }
             }
             path_cache.switch_back();
             self.run_list_read();
         } else {
             let as_read = input.as_read.as_str();
             match as_read {
                 "w" => {
                      // Cd the parent shell into the directory viewed by ls-key.
                      let path = self.list.relative_parent_dir_path;
                      let path = path.to_str().unwrap();
                      let cmd = format!(r#""$(printf 'cd {} \n ')""#, path).to_string();
                      terminal::parent_shell::type_text(cmd, 0);
                 },
                 "q" => (),
                 "fzf" => {
                     let mut path_cache = command_assistors::PathCache::new(
                         self.list.relative_parent_dir_path.as_path()
                     );
                     path_cache.switch();
                     let output = terminal::shell::cmd("fzf".to_string());
                     let file_path = output.unwrap();
                     terminal::shell::spawn("vim".to_string(), vec![file_path]);
                     path_cache.switch_back();
                     self.run_list_read();
                 },
                 "vim" => {
                     let mut path_cache = command_assistors::PathCache::new(
                         self.list.relative_parent_dir_path.as_path()
                     );
                     path_cache.switch();
                     //Split cmd ('vim')
                     let split: Vec<&str> = input.as_read.split("vim").collect();
                     let cmd = split.iter().last().unwrap();
                     let cmd = format!(r#"vim {}"#, cmd);
                     //let output = terminal::shell::cmd(cmd.clone());
                     //let file_path = output.unwrap();
                     terminal::shell::spawn("vim".to_string(), vec![]);
                     path_cache.switch_back();
                     self.run_list_read();
                 },
                 "zsh" => {
                     let mut path_cache = command_assistors::PathCache::new(
                         self.list.relative_parent_dir_path.as_path()
                     );
                     path_cache.switch();
                     //Split cmd ('zsh')
                     let split: Vec<&str> = input.as_read.split("zsh").collect();
                     let cmd = split.iter().last().unwrap();
                     let cmd = format!(r#"zsh {}"#, cmd);
                     //let output = terminal::shell::cmd(cmd.clone());
                     //let file_path = output.unwrap();
                     terminal::shell::spawn("zsh".to_string(), vec![]);
                     path_cache.switch_back();
                     self.run_list_read();
                 },
                 _ => {
                     let mut path_cache = command_assistors::PathCache::new(
                         self.list.relative_parent_dir_path.as_path()
                     );
                     path_cache.switch();
                     let output = terminal::shell::cmd(as_read.to_string()).unwrap();
                     path_cache.switch_back();
                     self.run_list_read();
                 }
             }
        }
    }

    fn readline_mode(mut self, list: List, input: Result<(Option<String>), std::io::Error>) {
        match input {
            Ok(t) =>  {
                if let Some(i) = t {
                    let input = Input::new();
                    let input = input.parse(i);
                    // Safe to unwrap.
                    match input.clone().cmd_type.unwrap() {
                        CmdType::cmd => {
                            self.cmd_mode(list, input);
                        },
                        CmdType::single_key => {
                            self.key_mode(list, input);
                        },
                        CmdType::multiple_keys => {
                            /*
                                * get_file_by_key for each key
                                * let text_vec = vec![r#"printf '1=file1; 2=file2;...'; \n "#]
                                * then type_text_spawn(text_vec);
                            */
                            self.return_file_by_key_mode(list, input);
                        }
                    }
                    ()
                } else {
                    ()
                }
            },
            Err(_) => ()
        }
    }

    // If you want to return the output of a commands, see fzf example below.
    // The commmand 'vim', without any args, is a special case handled by
    // ls-key. If the non-built in command doesn't return output and enters
    // into a child process (e.g. vim), then shell::cmd cannot be used, to my
    // understanding.
    fn run_cmd(mut self, list: List) {
        //let input = terminal::input_n_display::read();
        let input = self.clone().read_process_chars(&list);
        self.readline_mode(list, Ok(input));
    }

    fn read_process_chars(mut self, list: &List) -> Option<String> {
        let mut input: Vec<char> = vec![];
        let stdin = stdin();
        let stdout = stdout();
        let mut stdout = stdout.lock().into_raw_mode().unwrap();
        let mut stdin = stdin.lock();
        let mut result: Option<String> =  None;

        write!(stdout, "{}{}\n\r", termion::clear::CurrentLine, termion::cursor::Goto(1, 1)).unwrap();
        //write!(stdout,
        //    "{}{}",
        //   termion::clear::All,
        //   termion::cursor::Goto(1, 1),
        //).unwrap();
        //stdout.flush().unwrap();

        fn write(some_stuff: &[u8], stdout: &mut RawTerminal<StdoutLock>, input_string: String, locate: (u16, u16)) {
            //stdout.write_all(some_stuff).unwrap();
            //stdout.flush().unwrap();
            write!(
                stdout,
                "{}\n\r",std::str::from_utf8(&some_stuff).unwrap(),

            ).unwrap();
            //write!(stdout, "{}", termion::clear::CurrentLine).unwrap();
            write!(stdout,
                "{}{}{}{}", format!("{}", input_string.as_str()
                ),
               termion::clear::AfterCursor,
               termion::cursor::Goto(locate.0, locate.1),
               termion::cursor::Hide,
            ).unwrap();
        }

        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q') => break,
                Key::Char(c) => {
                    match c {
                        //' ' => {
                        //    println!("$")
                        //},
                        //'v' => println!("{}im", c),
                        _ => {
                            input.push(c);
                        }
                    }
                }
                Key::Alt(c) => println!("^{}", c),
                Key::Ctrl(c) => println!("*{}", c),
                Key::Esc => println!("ESC"),
                Key::Esc => println!("ESC"),
                Key::Left => println!("←"),
                Key::Right => println!("→"),
                Key::Up => println!("↑"),
                Key::Down => println!("↓"),
                Key::Backspace => {
                    if let Some(x) = input.pop() {
                    } else {
                        write!(stdout, "{}{}", termion::cursor::Goto(0, 2), termion::clear::AfterCursor).unwrap();
                    }
                },
                _ => {}
            }
            let input_string: String = input.iter().collect();
            let input_len = input.iter().count();
            let input_len = u16::try_from(input_len).ok().unwrap();
            let _first = input.iter().nth(0);
            if let Some(mut first) = _first {
                let key: Result<(usize), std::num::ParseIntError> = first.to_string().parse();
                if key.is_ok() {
                    first = &'r';
                } else {
                    if first != &'$' {
                         first = &'f';
                    }
                }

                let place = (1, 3);

                match first {
                    'f' => write(b"fuzzy-widdle mode detected...", &mut stdout, input_string.clone(), place),
                    'r' => write(b"return file mode detected...", &mut stdout, input_string.clone(), place),
                    '$' => write(b"command mode detected... ", &mut stdout, input_string.clone(), place),
                    _ => write(b"invalid mode detected...", &mut stdout, input_string.clone(), place),
                };
            }

            let show = self.display.clone();

            if let Some(x) = show {
                if x.0 == self.list.relative_parent_dir_path {
                    let display = str::replace(x.1.as_str(), "\n", "\n\r");
                    write(b"\n\r", &mut stdout, display.to_string(), (1, 1));
                }
            }

            stdout.flush().unwrap();

            if input.iter().last() == Some(&'\n') {
                input.pop();
                let input_string: String = input.iter().collect();
                result = Some(input_string);
                break
            }
        }

        //write!(stdout, "{}", termion::cursor::Show).unwrap();
        result
    }
}

#[derive(Debug, Clone, PartialEq)]
enum CmdType {
    single_key,
    multiple_keys,
    cmd,
}
#[derive(Debug, Clone, PartialEq, Default)]
struct Input {
    cmd: Option<String>,
    args: Option<Vec<String>>,
    as_read: String,
    cmd_type: Option<CmdType>
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
        let is_key = if args == None {
            let key: Result<(usize), std::num::ParseIntError> = cmd.clone().unwrap().parse();
            match key {
                Ok(_) => Some(true),
                Err(_) => Some(false)
            }
        } else {
            Some(false)
        };


        let are_all_keys = if let Some(c) = cmd.clone() {
             if c == "".to_string() {
                 let _args = args.clone();
                 if let Some(a) = _args {
                      self.are_all_keys(a)
                 }
                 else {
                     false
                 }
             } else {
                 false
             }
        } else {
            false
        };

        let cmd_type = if are_all_keys {
            CmdType::multiple_keys
        } else if let Some(k) = is_key {
            if k {
                CmdType::single_key
            } else {
                CmdType::cmd
            }
        } else {
            CmdType::cmd
        };

        self.cmd = cmd;
        self.args = args;
        self.as_read = input;
        self.cmd_type = Some(cmd_type);

        self
    }

    fn parse_cmd(&self, input: String) -> (Option<String>, Option<Vec<String>>) {
        let mut input: Vec<String> = input.clone().split(" ").map(|s| s.to_string()).collect();
        let cmd = input.remove(0);

        let args = self.defang_args(input);
        (Some(cmd), args)
     }

     fn are_all_keys(&self, input: Vec<String>) -> bool {
        let is_num = |x: &str| {
            let res: Result<(usize), std::num::ParseIntError> = x.parse();
            match res {
                Ok(_) => true,
                Err(_) => false
            }
        };
        let is_all_nums = !input.iter().any(|x| !is_num(x.as_str()));

        is_all_nums
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
    use super::{Input, LsKey, CmdType};


    #[test]
    #[ignore]//host
    fn parse() {
        let input = Input::new();
        let input = input.parse("vim Cargo.toml".to_string());

        assert_eq!(
           Some(CmdType::cmd),
           input.cmd_type
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
           Some(CmdType::cmd),
           input.cmd_type
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
           Some(CmdType::cmd),
           input.cmd_type
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
           Some(CmdType::single_key),
           input.cmd_type
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
           Some(CmdType::cmd),
           input.cmd_type
        );

        assert_eq!(
           Some("".to_string()),
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
             r#""$(printf 'q \n ')""#.to_string(),
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
             r#""$(printf 'q \n ')""#.to_string(),
        ];
        let spawn = super::terminal::parent_shell::type_text_spawn(text_vec, 200);
        //let spawn_quite = super::terminal::parent_shell::type_text_spawn(r#""$(printf ':q \n ')""#, 700);
        let ls_key = super::LsKey::new(path, true);
        ls_key.run_list_read();
        spawn.join();
        //spawn_quite.join();
    }
}
