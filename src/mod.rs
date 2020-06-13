pub mod list;
pub mod terminal;
pub mod fuzzy;

use std::convert::TryInto;
use std::path::{Path, PathBuf};
use std::fs::{create_dir_all, metadata, OpenOptions};
use std::io::prelude::*;
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
use sha2::{Sha256, Sha512, Digest};
use sha2::digest::generic_array::{ArrayLength, GenericArray};
use easy_hasher::easy_hasher::*;

pub mod app {
    use super::*;

    pub fn run<P: AsRef<Path>>(path: P, all: bool, test: bool) -> LsKey {
        if test {
            let mut path = path.as_ref().to_path_buf();
            create_dir_all(path.clone()).expect("Failed to create directories.");
            path.push(".lsk_test_output");
            let mut file = std::fs::File::create(path.clone()).unwrap();
        }
        let path = path.as_ref();
        let mut ls_key = LsKey::new(path, all, test);
        ls_key.run_list_read(ls_key.clone().is_fuzzed);
        let mut list = ls_key.list.clone();

        while ls_key.is_fuzzed {
            ls_key.list = list;
            let display = ls_key.display.clone();
            if let Some(fuzzy_list) = ls_key.fuzzy_list.clone() {
                let _list = ls_key.list;
                ls_key = LsKey::new(path, all, test);
                ls_key.list = fuzzy_list.clone();
                ls_key.display = display;
            } else if !ls_key.halt {
                let _list = ls_key.list;
                ls_key = LsKey::new(path, all, test);
                ls_key.list = _list;
                ls_key.display = display;
            }
            ls_key.run_list_read(ls_key.clone().is_fuzzed);
            list = ls_key.list.clone();
        }

        ls_key
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LsKey {
    pub list: List,
    pub all: bool,
    pub input: Option<String>,
    pub fuzzy_list: Option<List>,
    pub display: Option<(PathBuf, String)>,
    pub halt: bool,
    pub is_fuzzed: bool,
    pub test: bool,
    pub input_vec: Vec<String>,
    pub output_vec: Vec<String>
}

impl LsKey {
    pub fn new<P: AsRef<Path>>(path: P, all: bool, test: bool) -> Self {
        let mut ls_key: LsKey = Default::default();
        let list = if all {
           list::List::new(path)
               .list_include_hidden()
               .unwrap()
        } else {
           list::List::new(path)
               .list_skip_hidden()
               .unwrap()
        };

        ls_key.list = list;
        ls_key.all = all;
        ls_key.halt = true;
        ls_key.is_fuzzed = false;
        ls_key.test = test;

        ls_key
    }

    fn fuzzy_score(&mut self, mut input: String) -> fuzzy::demo::Scores {
        let files = &self.list.files;
        let dirs = &self.list.dirs;

        let mut input_vec_str: Vec<&str> = input.split(" ").collect();
        //let mut input_vec: Vec<String> = vec![];
        //for x in input_vec_str.iter() {
        //     input_vec.push(x.to_string())
        //
        //}

        if input_vec_str.iter().count() > 1{
            input_vec_str.pop();
            input = input_vec_str.into_iter().collect();
        }
        let score_list = |file: PathBuf| {
            (
             file.clone(),
             fuzzy::demo::score(file.to_str().unwrap(), &input)
            )
        };

        let files_score: Vec<fuzzy::demo::Score> =
           files.iter()
               .map(|file| fuzzy::demo::Score::Files(score_list(file.to_path_buf())))
               .collect();

        let dirs_score: Vec<fuzzy::demo::Score> =
           dirs.iter()
               .map(|dir| fuzzy::demo::Score::Dirs(score_list(dir.to_path_buf())))
               .collect();
           //list.map(|x

        let files = files_score;
        let dirs = dirs_score;

        fuzzy::demo::Scores {
            files,
            dirs
        }
    }

    fn fuzzy_rank(&mut self, mut scores: fuzzy::demo::Scores) -> fuzzy::demo::Scores {
        scores.files.sort_by(|a, b| fuzzy::demo::order(a, b));
        scores.dirs.sort_by(|a, b| fuzzy::demo::order(a, b));

        scores
    }

    // Filter out no-scores.
    fn fuzzy_filter(&mut self, mut scores: fuzzy::demo::Scores) -> fuzzy::demo::Scores {
         let mut files_vec: Vec<fuzzy::demo::Score> = vec![];
         let mut dirs_vec: Vec<fuzzy::demo::Score> = vec![];
         for score in scores.files.iter() {
             let path = score.score().0;
             let score = score.score().1;

             let thing = (path, score.clone());

             if score.is_some() {
                  files_vec.push(fuzzy::demo::Score::Files(thing));
             }
         }

         for score in scores.dirs.iter() {
             let path = score.score().0;
             let score = score.score().1;

             let thing = (path, score.clone());

             if score.is_some() {
                  dirs_vec.push(fuzzy::demo::Score::Dirs(thing));
             }
         }

         fuzzy::demo::Scores {
             files: files_vec,
             dirs: dirs_vec
         }
    }

    pub fn fuzzy_update(&mut self, input: String) -> Self {
        let scores = self.fuzzy_score(input);
        let scores = self.fuzzy_rank(scores);
        let scores = self.fuzzy_filter(scores);
        let list = self.scores_to_list(scores);
        let res =  self.fuzzy_update_list_read(&list);
        //self.list = list;
        self.display = res;
        self.fuzzy_list = Some(list);

        self.clone()
    }

    pub fn scores_to_list(&mut self, mut scores: fuzzy::demo::Scores) -> list::List {
        let files_list: Vec<PathBuf> = scores.files.iter().map(|score|
            score.score().0
        ).collect();
        let dirs_list: Vec<PathBuf> = scores.dirs.iter().map(|score|
            score.score().0
        ).collect();

        let pre_fuzzy = self.list.clone();

        self.list.files = files_list;
        self.list.dirs = dirs_list;

        //assert_ne!(pre_fuzzy, self.list);
        self.list.clone()
    }

    pub fn update(&mut self, list: List) {
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
    }

   pub fn run_list_read_beta(&mut self) {
            let list = self.list.clone();
            let entries: Vec<PathBuf> = list::order_and_sort_list(&list, true);

            let entries_keyed: Vec<String> = list::key_entries(entries);
            //let res = terminal::input_n_display::grid(entries_keyed);
            let res = terminal::input_n_display::grid(entries_keyed);
            let mut show = "".to_string();
            if let Some(r) = res {
                let grid = r.0;
                let width = r.1;
                let display = grid.fit_into_width(width);
                if display.is_some() && !self.test {
                     //println!("\n\n{}", d);
                     self.display = Some((self.list.parent_path.clone(), display.unwrap().to_string())); // safe to unwrap
                } else {
                    let display = grid.fit_into_columns(1);
                    self.display = Some((self.list.parent_path.clone(), display.to_string()));
                    //println!("\n\n");
                    //list::print_list_with_keys(list.clone());
                }
            } else {
                //println!("\n\n");
                //list::print_list_with_keys(list.clone());
            }
    }

   pub fn run_list_read(&mut self, halt: bool) {
            let list = self.list.clone();
            let entries: Vec<PathBuf> = list::order_and_sort_list(&list, true);

            let entries_keyed: Vec<String> = list::key_entries(entries);
            //let res = terminal::input_n_display::grid(entries_keyed);
            let res = terminal::input_n_display::grid(entries_keyed);
            let mut show = "".to_string();
            if let Some(r) = res {
                let grid = r.0;
                let width = r.1;
                let display = grid.fit_into_width(width);
                if display.is_some() && !self.test {
                     //println!("\n\n{}", d);
                     self.display = Some((self.list.parent_path.clone(), display.unwrap().to_string())); // safe to unwrap
                } else {
                    let display = grid.fit_into_columns(1);
                     self.display = Some((self.list.parent_path.clone(), display.to_string()));
                    //println!("\n\n");
                    //list::print_list_with_keys(list.clone());
                }
            } else {
                //println!("\n\n");
                //list::print_list_with_keys(list.clone());
            }

            if !halt {
                self.run_cmd();
            }
    }

   pub fn fuzzy_update_list_read(&mut self, list: &List) -> Option<(PathBuf, String)> {
            let entries: Vec<PathBuf> = list::order_and_sort_list(&list, false);

            let entries_keyed: Vec<String> = list::key_entries(entries);
            //let res = terminal::input_n_display::grid(entries_keyed);
            //println!("{:#?}", entries_keyed);
            let res = terminal::input_n_display::grid(entries_keyed); // stops here!
            //println!("\n....made it!\n");
            let mut show = "".to_string();
            if let Some(r) = res {
                let grid = r.0;
                let width = r.1;
                let display = grid.fit_into_width(width);
                if display.is_some() && !self.test {
                     //println!("\n\n{}", d);
                     //println!("\nmade it!\n");
                     let old_display = self.display.clone();
                     //self.display = Some((self.list.parent_path.clone(), d.to_string()));
                     //assert_eq!(self.display, Some((PathBuf::from(""), "".to_string())));
                     //assert_ne!(old_display, self.display);
                     return Some((self.list.parent_path.clone(), display.unwrap().to_string())) // safe to unwrap.
                     //println!("{:#?}", self.display);
                } else {
                    let display = grid.fit_into_columns(1);
                    //self.display = Some((self.list.parent_path.clone(), display.to_string()));
                    //println!("\n\n");
                    //list::print_list_with_keys(list.clone());
                     return Some((self.list.parent_path.clone(), display.to_string()))
                }
            } else {
                //println!("\n\n");
                //list::print_list_with_keys(list.clone());
                self.display.clone()
            }
    }

    fn return_file_by_key_mode(&mut self, list: List, input: Input, is_fuzzed: bool) {
        let get_file = |key_string: String| {
             let key: usize = key_string.parse().unwrap();
             self.list.get_file_by_key(key, !is_fuzzed).unwrap()
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

    pub fn key_mode(&mut self, list: List, input: Input, is_fuzzed: bool) {
        let key: usize = input.cmd.unwrap().parse().unwrap();
        match key {
            0 => {
                 self.list.parent_path.pop();
                 let file_pathbuf = self.list.parent_path.clone();
                 self.list.parent_path.pop();
                 let list = self.list.clone().update(file_pathbuf);
                 self.update(list);
                 self.halt = false;
                 self.run_list_read(is_fuzzed);
            },
            _ => {
                  let file_pathbuf = list.get_file_by_key(key, !is_fuzzed).unwrap();
                  if metadata(file_pathbuf.clone()).unwrap().is_dir() {
                      let file_path =
                          file_pathbuf
                          .to_str().unwrap()
                          .to_string();

                      let list = self.list.clone().update(file_pathbuf);
                      self.update(list);
                      self.halt = false;
                      self.run_list_read(is_fuzzed);
                  } else {
                      let file_path =
                          file_pathbuf
                          .to_str().unwrap()
                          .to_string();
                      terminal::shell::spawn("vim".to_string(), vec![file_path]);
                      self.halt = false;
                      self.run_list_read(is_fuzzed);
                  }
            }
        }
    }

    fn cmd_mode(&mut self, input: Input) {
         let args = input.args;
         if let Some(a) = args {
             let args = a;
             // Unwrap is safe because is_key is not None and there are args.
             let cmd = input.cmd.unwrap();
             let mut path_cache = command_assistors::PathCache::new(
                 self.list.parent_path.as_path()
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
             //self.run_list_read();
         } else {
             let as_read = input.as_read.as_str();
             match as_read {
                 "q" => (),
                 "fzf" => {
                     let mut path_cache = command_assistors::PathCache::new(
                         self.list.parent_path.as_path()
                     );
                     path_cache.switch();
                     let output = terminal::shell::cmd("fzf".to_string());
                     let file_path = output.unwrap();
                     terminal::shell::spawn("vim".to_string(), vec![file_path]);
                     path_cache.switch_back();
                     //self.run_list_read();
                 },
                 "vim" => {
                     let mut path_cache = command_assistors::PathCache::new(
                         self.list.parent_path.as_path()
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
                     //self.run_list_read();
                 },
                 "zsh" => {
                     let mut path_cache = command_assistors::PathCache::new(
                         self.list.parent_path.as_path()
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
                     //self.run_list_read();
                 },
                 _ => {
                     let mut path_cache = command_assistors::PathCache::new(
                         self.list.parent_path.as_path()
                     );
                     path_cache.switch();
                     let output = terminal::shell::cmd(as_read.to_string()).unwrap();
                     path_cache.switch_back();
                     //self.run_list_read();
                 }
             }
        }
    }

    fn key_related_mode(&mut self, list: List, input: Result<(Option<String>), std::io::Error>, is_fuzzed: bool) {
        match input {
            Ok(t) =>  {
                if let Some(i) = t {
                    let input = Input::new();
                    let input = input.parse(i);
                    // Safe to unwrap.
                    match input.clone().cmd_type.unwrap() {
                        CmdType::single_key => {
                            self.key_mode(list, input, is_fuzzed);
                        },
                        CmdType::multiple_keys => {
                            /*
                                * get_file_by_key for each key
                                * let text_vec = vec![r#"printf '1=file1; 2=file2;...'; \n "#]
                                * then type_text_spawn(text_vec);
                            */
                            self.return_file_by_key_mode(list, input, is_fuzzed);
                        },
                        _ => ()
                    }
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
    fn run_cmd(&mut self) {
        //If the is a fuzzy re-entry, we must reset is_fuzzed and halt to default.
        let mut execute = false;
        while !execute {
           let (some_list, input, _is_fuzzed, _execute) = self.read_process_chars(self.list.clone());
               execute = _execute;
               if let Some(list) = some_list {
                   if execute {
                       let new_list = list.clone();
                       self.key_related_mode(list, Ok(input), self.is_fuzzed);
                   } else {
                       break
                    }

               }
        }
    }

    fn test_data_update(&mut self, input: Option<String>) {
        if self.test == true {
            if input.is_some() {
                let mut hasher = Sha256::new();
                let hash = sha256(&input.clone().unwrap());
                //self.output_vec.push(hash.to_hex_string());

                let original_dir = self.clone().list.path_history.into_iter().nth(0);
                if original_dir.is_some() {
                    let mut original_dir = original_dir.unwrap();
                    //file.write_all(stuff.as_bytes()).unwrap();
                    original_dir.push(".lsk_test_output");
                    let mut file = OpenOptions::new()
                       .write(true)
                       .append(true)
                       .open(original_dir.clone().into_os_string().into_string().unwrap())
                       .unwrap();

                   if let Err(e) = writeln!(file, "{}", hash.to_hex_string()) {
                       eprintln!("Couldn't write to file: {}", e);
                   }
                }
            }
            if self.display.is_some() {
                let mut hasher = Sha256::new();
                let mut hasher = Sha256::new();
                let hash = sha256(&self.display.clone().unwrap().1);
                //self.output_vec.push(hash.to_hex_string());

                let original_dir = self.clone().list.path_history.into_iter().nth(0);
                if original_dir.is_some() {
                    let mut original_dir = original_dir.unwrap();
                    //file.write_all(stuff.as_bytes()).unwrap();
                    original_dir.push(".lsk_test_output");
                    let mut file = OpenOptions::new()
                       .write(true)
                       .append(true)
                       .open(original_dir.clone().into_os_string().into_string().unwrap())
                       .unwrap();

                   if let Err(e) = writeln!(file, "{}", hash.to_hex_string()) {
                       eprintln!("Couldn't write to file: {}", e);
                   }
                }
            }
        }
    }

    //fn test_data_sum_to_single_hash(&mut self) -> [u8; 32] {
    //    let mut complete_vec = self.input_vec.to_owned();
    //    complete_vec.append(&mut self.output_vec);
    //    let mut hasher = Sha256::new();
    //    for i in complete_vec.iter() {
    //        let str_i = std::str::from_utf8(i).unwrap();
    //        hasher.input(str_i);
    //    }

    //    let result: [u8; 32] = hasher.result().as_slice().try_into().expect("Wrong length");

    //    result
    //}

    fn read_process_chars(&mut self, list: List) -> (Option<list::List>, Option<String>, bool, bool) {
        let mut input:Input = Input::new();
        let stdin = stdin();
        let stdout = stdout();
        let mut stdout = stdout.lock().into_raw_mode().unwrap();
        let mut stdin = stdin.lock();
        let mut result: Option<String> =  None;
        let mut is_fuzzed = false;
        let mut the_list: Option<list::List> = None;

        clear_display(&mut stdout);

        let mut input_string: String = input.display.iter().collect();
        self.test_data_update(Some(input_string));
        display_files(self.clone(), b"", &mut stdout, (0, 3));

        for c in stdin.keys() {
            clear_display(&mut stdout);

            input = input.clone().match_event(c.unwrap());
            let mut input_string: String = input.display.iter().collect();
            let input_len = input.display.iter().count();
            let input_len = u16::try_from(input_len).ok().unwrap();
            let _first = input.display.iter().nth(0);
            let last = input.display.iter().last();

            let place = (0, 1);
            if let Some(mut first) = _first {
                self.test_data_update(Some(input_string.clone()));
                display_input(input_string.clone(), &mut stdout, place);

                let key: Result<(usize), std::num::ParseIntError> = first.to_string().parse();
                if key.is_ok() {
                    first = &'r';
                }

                let some_mode = mode_parse(input_string.clone());

                if let Some(mode) = some_mode {
                    match mode {
                        Mode::Cmd(cmd_mode_input) => {
                             if last == Some(&'\n') {
                                 let cmd_res = cmd_read(&mut input.display, self);
                                 input.display = cmd_res.0;
                                 input_string = cmd_res.1;
                             }
                        },
                        Mode::Work => {
                            let few_ms = std::time::Duration::from_millis(2000);
                             if last == Some(&'\n') {
                                 let path = self.list.parent_path.clone();
                                 let path = path.to_str().unwrap();
                                 let cmd = format!(r#""$(printf 'cd {} \n ')""#, path).to_string();
                                 terminal::parent_shell::type_text(cmd, 0);
                                 self.is_fuzzed = false;
                                 break
                            } else {

                            }
                        },
                        Mode::Fuzzy(fuzzy_mode_input) => {
                            let _show = self.display.clone();
                            let some_keys = parse_keys(fuzzy_mode_input.as_str());

                            if let Some(keys) = some_keys {
                                if let Some(x) = self.fuzzy_list.clone() {
                                    //self.list = x.clone();
                                    input_string = keys;
                                    input.display = input_string.chars().collect();
                                    the_list = Some(x.clone());
                                    // clear input and drop in the parsed key.
                                }
                            } else {

                                if input.display.iter().last() != Some(&'\n') {
                                    let ls_key = self.fuzzy_update(fuzzy_mode_input);
                                }
                            }

                            is_fuzzed = true;
                        },
                        _ => {}
                    }
                }
            }

            self.test_data_update(Some(input_string.clone()));
            display_files(self.clone(), b"", &mut stdout, (0, 3));

            if input.display.iter().last() == Some(&'\n') {
                input.display.pop();
                let input_string: String = input.display.iter().collect();
                result = Some(input_string);
                self.is_fuzzed = is_fuzzed;
                if self.is_fuzzed {
                }
                break
            }
        }

        if the_list.is_none() {
            the_list = Some(self.list.clone());
        }

        (the_list, result, is_fuzzed, input.execute)
    }
}

fn cmd_read(input: &mut Vec<char>, ls_key: &mut LsKey) -> (Vec<char>, String) {
     input.pop();
     let input_string: String = input.iter().collect();
     let cmd_mode = mode_parse(input_string.clone()).unwrap(); //safe
     match cmd_mode {
         Mode::Cmd(cmd_mode_input) => {
             let input = Input::new();
             let input = input.parse(cmd_mode_input);

             match input.clone().cmd_type.unwrap() {
                 CmdType::cmd => {
                     ls_key.cmd_mode(input);
                     ls_key.run_list_read_beta();
                 },
                 _ => {}
             }
             //break
         }
         _ => { }
     }

     (input.to_vec(), input_string)
}

fn clear_display(stdout: &mut RawTerminal<StdoutLock>) {
    write!(
        stdout,
        "{}",
        termion::clear::All
    ).unwrap();
    stdout.flush().unwrap();
}

fn display_input(input_string: String, stdout: &mut RawTerminal<StdoutLock>, position: (u16, u16)) {
    write!(stdout,
        "{}{}{}{}", format!("{}", input_string.as_str()
        ),
       termion::clear::AfterCursor,
       termion::cursor::Goto((position.0), (position.1 + 1)),
       termion::cursor::Hide,
    ).unwrap();
    stdout.flush().unwrap();
}

fn display_files(ls_key: LsKey, some_stuff: &[u8], stdout: &mut RawTerminal<StdoutLock>, position: (u16, u16)) {
     let show = ls_key.clone().display;
     if let Some(x) = show {
         if x.0 == ls_key.list.parent_path {
              //into_raw_mode requires carriage returns.
              let display = str::replace(x.1.as_str(), "\n", "\n\r");
              write!(
                  stdout,
                  "{}{}{}\n", format!("{}", std::str::from_utf8(&some_stuff).unwrap()),
                  termion::cursor::Goto(position.0, position.1),
                  termion::cursor::Hide,

              ).unwrap();
              stdout.flush().unwrap();

              write!(stdout,
                  "{}{}{}{}", format!("{}", display.as_str()
                  ),
                 termion::clear::AfterCursor,
                 termion::cursor::Goto((position.0), (position.1 + 1)),
                 termion::cursor::Hide,
              ).unwrap();
              stdout.flush().unwrap();

              write!(
                  stdout,
                  "{}",
                  termion::cursor::Goto(0, 3),
              ).unwrap();
              stdout.flush().unwrap();
         }
     }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CmdType {
    single_key,
    multiple_keys,
    cmd,
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Input {
    pub cmd: Option<String>,
    pub args: Option<Vec<String>>,
    pub as_read: String,
    pub cmd_type: Option<CmdType>,
    pub display: Vec<char>,
    pub execute: bool,
    pub unwiddle: bool, //i.e. backspacing
}



impl Input {
    pub fn new() -> Self {
        let input: Input = Default::default();
        let mut input: Input = Default::default();
        input.execute = true;
        input.unwiddle = false;

        input
    }

    pub fn match_event(mut self, c: termion::event::Key) -> Self {
            self.unwiddle = false;
            match c {
                Key::Char(c) => {
                    match c {
                        _ => {
                            self.display.push(c);
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
                    self.unwiddle = true;
                    if let Some(x) = self.display.pop() {
                        if self.display.iter().count() == 0 {
                            self.execute = false;
                        }
                    }
                },
                _ => {}
            }
            self
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

    pub fn parse(mut self, input: String) -> Self {
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
             if c == "r".to_string() {
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

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Fuzzy(String),
    Cmd(String),
    Work,
}

pub fn mode_parse(mut input: String) -> Option<Mode> {
    let len = input.len();
    let mode = if len > 2 {
         let mode: String = input.drain(..2).collect();
         let mode = mode.as_str();
         let fuzzy = "f ";
         let cmd = "c ";
         match mode {
             "f " => Some(Mode::Fuzzy(input.clone())),
             "c " => Some(Mode::Cmd(input.clone())),
             _ => None
         }
    } else if len == 2 {
         let mode: String = input.drain(..1).collect();
         let mode = mode.as_str();
         match mode {
             "w" => Some(Mode::Work),
             _ => None
         }
    } else {
        None
    };

    mode
}

fn parse_keys(input: &str) -> Option<String> {
    let x = input;
    let mut y: Vec<&str> = x.split(" ").collect();
    let mut count = y.iter().count();

    let some = if count > 1 {
    let z = y.iter().nth(0).unwrap();
    //let index = |z: &str| -> usize {
    //     y.iter().position(|x| x == &z).unwrap()
    //};
    y.remove(0);

    let mut n = 0;
    let single_key: bool;
    if count == 2 {
        single_key = true;
    } else {
        single_key = false;
    }
    loop {
        y.insert(n, " ");
        count = count - 1;
        n = n + 2;
        if count == 1 {
            break
        }
    }

    if single_key {
       y.remove(0);
    }

    let z: String = y.into_iter().collect();

    if z == "".to_string() {
        None
    } else {
        Some(z)
    }

    } else {
        None
    };

    some
}

#[cfg(test)]
mod app_test {
    use std::fs::{File, metadata,};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::env;
    use fixture::{Fixture, command_assistors};
    use super::{Input, LsKey, CmdType, Mode, mode_parse};
    use super::*;

    macro_rules! test {
        (
            $list_all_bool: expr, // lk -a would be true
            $name:ident,
            $test_file_path :expr, // We write text to a file so we know it's it when it's opened in test.
            $delay: expr, // Delay in each character typed.
            $input1: expr,
            $input2: expr,
            $input3: expr,
            $input4: expr,
            $input5: expr,
            $input6: expr,
            $input7: expr,
            $sub_path: expr, //We test all of this in a specific path. This'll create a sub-dir under that test-path.
            $intent: expr, //Explain what will happen in the test so tester can visually verify.
            $file_hash: expr,
            $test_macro: ident //We want to ignore the tests when we want and run when we want.
        ) => {

            #[test]
            #[$test_macro]
            fn $name() {
                let path = format!("/tmp/lsk_tests/{}/", $sub_path);

                let mut fixture = Fixture::new()
                    .add_dirpath(path.clone())
                    .add_dirpath(format!("{}basilides/", path.clone()))
                    .add_dirpath(format!("{}cyrinus/", path.clone()))
                    .add_dirpath(format!("{}nabor/", path.clone()))
                    .add_dirpath(format!("{}nazarius/", path.clone()))
                    .add_dirpath(format!("{}primus/", path.clone()))
                    .add_dirpath(format!("{}felician/", path.clone()))
                    .add_dirpath(format!("{}marcelinus/", path.clone()))
                    .add_dirpath(format!("{}isidore/", path.clone()))
                    .add_dirpath(format!("{}margaret/", path.clone()))
                    .add_dirpath(format!("{}angela/", path.clone()))
                    .add_dirpath(format!("{}francis/", path.clone()))
                    .add_dirpath(format!("{}gregory/", path.clone()))
                    .add_dirpath(format!("{}joseph/", path.clone()))
                    .add_dirpath(format!("{}anne/", path.clone()))
                    .add_dirpath(format!("{}joachim/", path.clone()))
                    .add_dirpath(format!("{}faustina/", path.clone()))
                    .add_dirpath(format!("{}john/", path.clone()))
                    .add_dirpath(format!("{}peter/", path.clone()))
                    .add_dirpath(format!("{}cecilia/", path.clone()))
                    .add_dirpath(format!("{}rita/", path.clone()))
                    .add_dirpath(format!("{}magdelene/", path.clone()))
                    .add_dirpath(format!("{}expeditus/", path.clone()))
                    .add_dirpath(format!("{}sebastian/", path.clone()))
                    .add_dirpath(format!("{}gabriel/", path.clone()))
                    .add_dirpath(format!("{}michael/", path.clone()))
                    .add_dirpath(format!("{}jude/", path.clone()))
                    .add_dirpath(format!("{}anthony/", path.clone()))
                    .add_dirpath(format!("{}nicholaus/", path.clone()))
                    .add_dirpath(format!("{}teresa/", path.clone()))
                    .build();

                let path_path = Path::new(path.clone().as_str()).to_path_buf();

                let mut sample_files_files = PathBuf::from(".fixtures");
                sample_files_files.push("sample_files");

                let mut path_test = path_path.clone();
                path_test.push("sample_files");

                let md = metadata(path_test.clone());
                let test_path_string = path_test.clone().into_os_string().into_string().unwrap();

                if !md.is_ok() {
                    fixture.build();
                    Command::new("cp")
                        .arg("-r".to_string())
                        .arg(sample_files_files.clone().into_os_string().into_string().unwrap())
                        .arg(test_path_string.clone())
                        .output()
                        .expect("failed to execute lsk process");
                }

                let mut path_cache = command_assistors::PathCache::new(&path_test);
                // Changing directories.
                path_cache.switch();

                println!("");
                let text_vec = vec![
                     format!(r#""{}""#, $input1),
                     format!(r#""{}""#, $input2),
                     format!(r#""{}""#, $input3),
                     format!(r#""{}""#, $input4),
                     format!(r#""{}""#, $input5),
                     format!(r#""{}""#, $input6),
                     format!(r#""{}""#, $input7),
                ];

                //println!("\n\n\nNew case intent:\n{}", $intent);
                //let few_ms = std::time::Duration::from_millis(5000);
                //std::thread::sleep(few_ms);

                let spawn = super::terminal::parent_shell::type_text_spawn(text_vec, $delay);
                let mut ls_key = super::app::run(test_path_string.clone(), $list_all_bool, true);
                spawn.join();

                let mut test_output_path = path_path.clone();
                test_output_path.push("sample_files");
                test_output_path.push(".lsk_test_output");
                let mut test_output_path_string = test_output_path.clone().into_os_string().into_string().unwrap();
                let mut output_mv_to_path = path_path.clone();
                let mut output_mv_to_path_string = path_path.clone().into_os_string().into_string().unwrap();

                Command::new("mv")
                    .arg(test_output_path_string)
                    .arg(output_mv_to_path_string.clone())
                    .output()
                    .expect("failed to execute lsk process");

                let few_ms = std::time::Duration::from_millis(100);
                std::thread::sleep(few_ms);

                let mut output_mv_to_path = path_path.clone();
                output_mv_to_path.push(".lsk_test_output");
                let mut output_mv_to_path_string = output_mv_to_path.clone().into_os_string().into_string().unwrap();

                println!("\npath:\n{}", output_mv_to_path_string.clone());

                let file256 = file_sha256(output_mv_to_path_string.clone());
                let hash: Hash;

                match file256 {
                    Ok(h) => {
                        assert_eq!(
                            h.to_hex_string(),
                            $file_hash.to_string()
                        )
                    },
                    Err(..) => assert!(false)
                }

                path_cache.switch_back();

                std::fs::remove_file(output_mv_to_path_string).unwrap();
            }
        };
    }

    test!(
          false, //list_all_bool
          macro_enter_file,
          "Makefile",
          100,               //$delay in milleseconds
          "$(printf '3\r')", //$input1
          "$(printf ':q\r')",//$input2
          "$(printf 'q\r')", //$input3
          "",                //$input4
          "",                //$input5
          "",                //$input6
          "",                //$input7
          "macro_enter_file",
          ">Run lsk\n>Open file by key (2)\n>Quite vim\n>Quite lsk",
          "e636b86d6467fc7880254f18611971bb9f04e9d7f1414dd6bd1c13ead34b6b25",
          ignore/*macro_use*/
    );

    test!(
          true, //list_all_bool
          macro_enter_file_list_all,
          ".eternal",
          100,               //$delay in milleseconds
          "$(printf '2\r')", //$input1
          "$(printf ':q\r')",//$input2
          "$(printf 'q\r')", //$input3
          "",                //$input4
          "",                //$input5
          "",                //$input6
          "",                //$input7
          "macro_enter_file_list_all",
          ">Run lsk\n>Open hidden file by key (2)\n>Quite vim\n>Quite lsk",
          "488a19bb1d0fdbefa492333e3b54f772ef5b5f2547e64f9e062cd81f6f48f34a",
          ignore/*macro_use*/
    );

    test!(
          false,
          macro_fuzzy_enter_file,
          "intercession",
          100,               //$delay in milleseconds
          "$(printf 'f boo\r')",
          "$(printf '4\r')",
          "$(printf ':q\r')",
          "$(printf 'q\r')",
          "",
          "",
          "",
          "macro_fuzzy_enter_file",
          ">Run lsk\n>Fuzzy widdle\n>Open file by key (1)\n>Quite vim\n>Quite lsk",
          "cf5e09bfcb83e3d33c459488862a08b0f4a255531fea3aa7dcc0b16805ecd934",
          ignore/*macro_use*/
    );

    test!(
         false,
          macro_fuzzy_enter_dir,
          "a-file",
          100,               //inrease 200 => 500 ms to see better.
          "$(printf 'f ins\r')",
          "$(printf '5\r')",
          "$(printf 'q\r')",
          "",
          "",
          "",
          "",
          "macro_fuzzy_enter_dir",
          ">Run lsk\n>Fuzzy widdle\n>Open dir by key (1)\n>Quite vim\n>Quite lsk",
          "d3c43dc3b99ba0d23060fc9f7a233dad1282c3ebf265253d26f13be019a1ce41",
          ignore/*macro_use*/
    );

    test!(
         false,
          macro_fuzzy_enter_dir_go_back_then_repeat,
          "a-file",
          100,               //inrease 200 => 500 ms to see better.
          "$(printf 'f do\r')",
          "$(printf '2\r')",
          "$(printf '0\r')",
          "$(printf 'f do\r')",
          "$(printf '2\r')",
          "$(printf 'q\r')",
          "",
          "macro_fuzzy_enter_dir",
          ">Run lsk\n>Fuzzy widdle\n>Open dir by key (1)\n>Go back (0) and repeat\n>Quite vim\n>Quite lsk",
          "1e137b3ad8fffa9bc9011d808c70069edeca6904edd976ea13efe694b71a408a",
          ignore/*macro_use*/
    );

    test!(
         false,
          macro_go_back_fuzzy_enter_back_into_dir,
          "a-file",
          100,               //inrease 200 => 500 ms to see better.
          "$(printf '0\r')",
          "$(printf 'f sa\r')",
          "$(printf '2\r')",
          "$(printf 'q\r')",
          "",
          "",
          "",
          "macro_go_back_fuzzy_enter_back_into_dir",
          ">Run lsk\n>Go back (0)\n>Fuzzy widdle\n>Open back into original dir by key (2)\n>\n>Quite lsk",
          "5fb1505474abce6d3c2797fdb1777746bf78b29282f398b97a087cf278bcbc60",
          ignore/*macro_use*/
    );

    test!(
         false,
          macro_walk_in_park,
          "a-file",
          100,               //inrease 200 => 500 ms to see better.
          "$(printf '24\r')",
          "$(printf '1\r')",
          "$(printf 'f con\r')",
          "$(printf '2\r')",
          "$(printf '5\r')",
          "$(printf ':q\r')",
          "$(printf 'q\r')",
          "macro_walk_in_park",
          ">Run lsk\n>Go back (0)\n>Fuzzy widdle\n>Open back into original dir by key (2)\n>\n>Quite lsk",
          "c3e28f308c901173a895e437b14383121df6991f7ccec2a763332c43ea4c7108",
          ignore/*macro_use*/
    );

     test!(
           false, //list_all_bool
           macro_bad_fuzzy_backspace,
           "Makefile",
           100,
           "f itf",
           "BackSpace",
           "BackSpace",
           "BackSpace",
           "BackSpace",
           "BackSpace",
           "q\rq\r",
           "macro_bad_fuzzy_backspace",
           ">Run lsk\n>OFuzzy widdle (2)\n>Backspace fully (bad behavior)\n>Quite lsk",
           "dd3c81714c9579295316517bdb6a7d28d59d6cbe6352ad7a725178b17abebca3",
          ignore/*macro_use*/
     );

    #[test]
    #[ignore]//docker
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
    #[ignore]//docker
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
    #[ignore]//docker
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
    #[ignore]//docker
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
    #[ignore]//docker
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
    #[ignore]//docker
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
    #[ignore]//docker
    fn test_mode_parse() {
       let input_single = "f something".to_string();
       let some_fuzzy_search_single = mode_parse(input_single.clone());

       let input_multi = "f something and more".to_string();
       let some_fuzzy_search_multi = mode_parse(input_multi.clone());

       let input_lack = "f ".to_string();
       let some_fuzzy_search_lack = mode_parse(input_lack.clone());

       let input_lack_more = "f".to_string();
       let some_fuzzy_search_lack_more = mode_parse(input_lack_more.clone());

       let input_wrong = "d something".to_string();
       let some_fuzzy_search_wrong = mode_parse(input_wrong.clone());

       assert_eq!(
           some_fuzzy_search_lack,
           None
       );

       assert_eq!(
           some_fuzzy_search_lack_more,
           None
       );

       assert_eq!(
           some_fuzzy_search_single,
           Some(Mode::Fuzzy("something".to_string()))
       );

       assert_eq!(
           some_fuzzy_search_multi,
           Some(Mode::Fuzzy("something and more".to_string()))

       );

       assert_eq!(
           some_fuzzy_search_wrong,
           None
       );
    }
}
