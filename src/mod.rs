pub mod list;
pub mod terminal;

use list::Entry;

use std::path::{Path, PathBuf};
use std::fs::{create_dir_all, metadata, OpenOptions};
use list::List;
use fixture::command_assistors;
use termion::input::TermRead;
use termion::event::Key;
use termion::raw::{IntoRawMode, RawTerminal};
use std::io::{ Write, stdout, stdin, StdoutLock};
use termion::screen::AlternateScreen;
use easy_hasher::easy_hasher::*;

pub mod app {
    use super::*;

    pub fn run<P: AsRef<Path>>(path: P, all: bool, test: bool, fzf_hook_path: Option<PathBuf>, fzc_hook_path: Option<PathBuf>, fzd_hook_path: Option<PathBuf>) -> LsKey {
        if test {
            let mut path = path.as_ref().to_path_buf();
            create_dir_all(&path).expect("Failed to create directories.");
            path.push(".lsk_test_output");
            std::fs::File::create(&path).expect("failed to create lsk output file");
        }
        let path = path.as_ref();
        let mut ls_key = LsKey::new(path, all, test, fzf_hook_path.clone(), fzc_hook_path.clone(), fzd_hook_path.clone());
        ls_key.update_file_display(ls_key.is_fuzzed, false);
        ls_key.run_cmd();
        let mut list = ls_key.list.clone();

        while ls_key.is_fuzzed {
            ls_key.list = list;
            let display = ls_key.display.clone();
            if let Some(fuzzy_list) = ls_key.fuzzy_list.clone() {
                let _list = ls_key.list;
                ls_key = LsKey::new(path, all, test, fzf_hook_path.clone(), fzc_hook_path.clone(), fzd_hook_path.clone());
                ls_key.list = fuzzy_list.clone();
                ls_key.display = display;
            } else if !ls_key.halt {
                let _list = ls_key.list;
                ls_key = LsKey::new(path, all, test, fzf_hook_path.clone(), fzc_hook_path.clone(), fzd_hook_path.clone());
                ls_key.list = _list;
                ls_key.display = display;
            }
            ls_key.update_file_display(ls_key.is_fuzzed, false);
            ls_key.run_cmd();
            list = ls_key.list.clone();
        }

        ls_key
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LsKey {
    pub list: List,
    pub all: bool,
    pub input: Input,
    pub fuzzy_list: Option<List>,
    pub pre_fuzz_list: Option<List>,
    pub display: Option<(PathBuf, String)>,
    pub halt: bool,
    pub is_fuzzed: bool,
    pub test: bool,
    pub input_vec: Vec<String>,
    pub output_vec: Vec<String>,
    pub fzf_hook_path: Option<PathBuf>,
    pub fzc_hook_path: Option<PathBuf>,
    pub fzd_hook_path: Option<PathBuf>,
}

impl LsKey {
    pub fn new<P: AsRef<Path>>(path: P, all: bool, test: bool, fzf_hook_path: Option<PathBuf>, fzc_hook_path: Option<PathBuf>, fzd_hook_path: Option<PathBuf>) -> Self {
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
        ls_key.fzf_hook_path = fzf_hook_path;
        ls_key.fzc_hook_path = fzc_hook_path;
        ls_key.fzd_hook_path = fzd_hook_path;

        ls_key
    }

    fn fuzzy_score(&mut self, mut input: String) -> list::fuzzy_score::Scores {
        let files = &self.list.files;

        let mut input_vec_str: Vec<&str> = input.split(" ").collect();

        if input_vec_str.iter().count() > 1{
            input_vec_str.pop();
            input = input_vec_str.into_iter().collect();
        }
        let score_list = |entry: Entry| {
            (
             entry.clone(),
             list::fuzzy_score::score(entry.path.to_str().unwrap(), &input)
            )
        };

        let files_score: Vec<list::fuzzy_score::Score> =
           files.iter()
               .map(|file| list::fuzzy_score::Score::Files(score_list(
                                Entry {
                                    path: file.path.to_path_buf(),
                                    file_type: file.file_type.clone(),
                                    key: file.key
                                }
                           )
                       )
                )
               .collect();

        let files = files_score;

        list::fuzzy_score::Scores {
            files,
        }
    }

    fn fuzzy_rank(&mut self, mut scores: list::fuzzy_score::Scores) -> list::fuzzy_score::Scores {
        scores.files.sort_by(|a, b| list::fuzzy_score::order(a, b));

        scores
    }

    // Filter out no-scores.
    fn fuzzy_filter(&mut self, scores: list::fuzzy_score::Scores) -> list::fuzzy_score::Scores {
         let mut files_vec: Vec<list::fuzzy_score::Score> = vec![];
         for score in scores.files.iter() {
             let path = score.score().0;
             let score = score.score().1;

             let thing = (path, score.clone());

             if score.is_some() {
                  files_vec.push(list::fuzzy_score::Score::Files(thing));
             }
         }

         list::fuzzy_score::Scores {
             files: files_vec,
         }
    }

    pub fn fuzzy_update(&mut self, input: String) -> Self {
        let scores = self.fuzzy_score(input);
        let scores = self.fuzzy_rank(scores);
        let scores = self.fuzzy_filter(scores);
        let list = self.scores_to_list(scores);
        self.update_file_display(true, false);
        self.fuzzy_list = Some(list);

        self.clone()
    }


    pub fn scores_to_list(&mut self, scores: list::fuzzy_score::Scores) -> list::List {
        let files_list: Vec<Entry> = scores.files.iter().map(|score|
            Entry {
                path: score.score().0.path,
                file_type: score.score().0.file_type,
                key: score.score().0.key
            }
        ).collect();

        self.list.files = files_list;

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

   pub fn update_file_display(&mut self, halt: bool, mut filter: bool) {
            let mut go = true;
            let entries = self.list.order_and_sort_list(true, filter);
            let entries_count = self.list.files.iter().count();
            let mut start = 0;
            let mut end = entries_count;
            if let Some(ls) = &self.list.filter {
                start = ls.clone().into_iter().nth(0).unwrap();
                end = *ls.into_iter().last().unwrap();
            } else {
            }
            while go {
                let entries = self.list.order_and_sort_list(true, filter);
                let mut entries_keyed: Vec<String> = list::key_entries(entries.clone());
                if end  < entries_count {
                    let last = format!("[{}...{}]", end + 1, entries_count);
                    entries_keyed.push(last);
                }
                let res = terminal::input_n_display::grid(entries_keyed.clone());
                if let Some(r) = res {
                    let grid = r.0;
                    let width = r.1;
                    let height = r.2;
                    let display: terminal::input_n_display::Display;
                    let _display = grid.fit_into_width(width);
                    let pad: usize;
                    if _display.is_some() && !self.test {
                         display = _display.unwrap(); // Safe to unwrap.
                         pad = 4;
                    } else {
                         display = grid.fit_into_columns(1);
                         pad = 5;
                    }
                    let grid_row_count = display.row_count();
                    if (grid_row_count + pad) > height {
                        //panic!("Can't fit list into screen.");
                        //
                        let range = start..end;

                        let mut filter_vec: Vec<usize> = vec![];

                        range.into_iter().for_each(|i|
                            filter_vec.push(i)
                        );

                        self.list.filter = Some(filter_vec);
                        filter = true;

                        end = end - 1;

                    } else {
                       go = false;
                    }

                    self.display = Some((self.list.parent_path.clone(), display.to_string()));
                } else {
                    go = false;
                }

            }
    }

    fn return_file_by_key_mode(&mut self, input: Input, is_fuzzed: bool) {
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
            let _output_vec: Vec<std::process::Output> =
                r.iter()
                    .map(|key|
                         format_cmd(PathBuf::from(key))
                    ).map(|statement|
                        format!(r#""$(printf '{} \n ')""#, statement)
                    ).map(|cmd|
                        terminal::parent_shell::type_text(
                            cmd,
                            0
                        )
                    ).collect();
        } else {
            ()
        }
    }

    pub fn filter_mode(&mut self, list: List) {
        let input_string: String = self.input.display.iter().collect();
        let mut input_vec_str: Vec<&str> = input_string.split("-").collect();
        let mut key_vec: Vec<usize> = vec![];

        // Does it end in "-"?
        let last = input_vec_str.iter().last();
        let mut open_range = false;
        if let Some(l) = last {
            if l == &"" {
                open_range = true;
           }
        }

        // Only want to deal with integers if it's an open range.
        if open_range {
           input_vec_str.pop();
        }

        // Make sure it'sall integers.
        for i in input_vec_str.into_iter() {
            let key: Result<usize, std::num::ParseIntError> = i.parse();
            if key.is_ok() {
                key_vec.push(key.unwrap());
            }

        }

        let end: usize;
        let start = key_vec.clone().into_iter().nth(0).unwrap();

        if open_range {
            end =  list.files.iter().count() + 1;
        } else {
            end = key_vec.clone().into_iter().nth(1).unwrap() + 1;
        }

        let range = start..end;

        let mut filter_vec: Vec<usize> = vec![];

        range.into_iter().for_each(|i|
            filter_vec.push(i)
        );

        self.list.filter = Some(filter_vec);
        self.update_file_display(false, true);
        self.run_cmd()
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
                 let halt = self.list.filter.is_some();
                 self.update_file_display(is_fuzzed, halt);
                 if !halt {
                     self.run_cmd();
                 }
            },
            _ => {
                  let file_pathbuf = list.get_file_by_key(key, !is_fuzzed).unwrap();
                  if metadata(file_pathbuf.clone()).unwrap().is_dir() {
                      let list = self.list.clone().update(file_pathbuf);
                      self.update(list);
                      self.halt = false;
                      let halt = self.list.filter.is_some();
                      self.update_file_display(is_fuzzed, halt);
                      if !halt {
                          self.run_cmd();
                      }
                  } else {
                      let file_path =
                          file_pathbuf
                          .to_str().unwrap()
                          .to_string();
                      terminal::shell::spawn("vim".to_string(), vec![file_path]);
                      self.halt = true;
                      self.update_file_display(is_fuzzed, self.halt);
                      self.run_cmd();
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
             let list_parent_path = self.list.parent_path.clone();
             let mut path_cache = command_assistors::PathCache::new(
                 list_parent_path.as_path()
             );
             path_cache.switch();
             match cmd.as_str() {
                 //"fzf" => {
                 //    //Split cmd ('fzf')
                 //    let split: Vec<&str> = input.as_read.split("fzf").collect();
                 //    let cmd = split.iter().last().unwrap();
                 //    let cmd = format!(r#"fzf {}"#, cmd);
                 //    let output = terminal::shell::cmd(cmd.clone());
                 //    let file_path = output.unwrap();
                 //    terminal::shell::spawn("vim".to_string(), vec![file_path]);
                 //},
                 //"fzc" => {
                 //    //let split: Vec<&str> = input.as_read.split("fzc").collect();
                 //    let fzc_pathbuf = self.fzc_hook_path.as_ref().expect("fzc fail: no fzc hook path specified");
                 //    let fzc_path_string = fzc_pathbuf.clone().into_os_string().into_string().unwrap();
                 //    let output = terminal::shell::output("sh".to_string(), vec![fzc_path_string]).expect("fail to get output from fzc hook script");
                 //    let cmd = String::from_utf8_lossy(&output.stdout).to_string();
                 //    let mut input = Input::new();
                 //    let input = input.parse(cmd);
                 //    assert!(input.args.is_some());
                 //    self.cmd_mode(input);
                 //},
                 _ => {
                      terminal::shell::spawn(cmd.to_string(), args);
                 }
             }
             path_cache.switch_back();
         } else {
             let as_read = input.as_read.as_str();
             match as_read {
                 "q" => (),
                 "fzf" => {
                     let mut path_cache = command_assistors::PathCache::new(
                         self.list.parent_path.as_path()
                     );
                     path_cache.switch();
                     let fzf_pathbuf = self.fzf_hook_path.as_ref().expect("fzf fail: no fzf hook path specified");
                     let fzf_path_string = fzf_pathbuf.clone().into_os_string().into_string().unwrap();
                     let file_path = terminal::shell::cmd(fzf_path_string).unwrap();
                     //assert!(input.args.is_some());
                     //let cmd_res = terminal::shell::cmd(cmd_path).unwrap();
                     terminal::shell::spawn("vim".to_string(), vec![file_path]);
                     path_cache.switch_back();
                 },
                 "fzc" => {
                     let mut path_cache = command_assistors::PathCache::new(
                         self.list.parent_path.as_path()
                     );
                     path_cache.switch();
                     let fzc_pathbuf = self.fzc_hook_path.as_ref().expect("fzc fail: no fzc hook path specified");
                     let fzc_path_string = fzc_pathbuf.clone().into_os_string().into_string().unwrap();
                     let cmd = terminal::shell::cmd(fzc_path_string).unwrap();
                     let mut input = Input::new();
                     let input = input.parse(cmd);
                     //assert!(input.args.is_some());
                     //let cmd_res = terminal::shell::cmd(cmd_path).unwrap();
                     path_cache.switch_back();
                     self.cmd_mode(input);
                 },
                 "fzd" => {
                     let list_parent_path = self.list.parent_path.clone();
                     let mut path_cache = command_assistors::PathCache::new(
                         list_parent_path.as_path()
                     );
                     path_cache.switch();
                     let fzd_pathbuf = self.fzd_hook_path.as_ref().expect("fzd fail: no fzd hook path specified");
                     let fzd_path_string = fzd_pathbuf.clone().into_os_string().into_string().unwrap();
                     let dir = terminal::shell::cmd(fzd_path_string).unwrap();
                     let mut dir_pathbuf = PathBuf::from(dir);
                     let is_fuzzed = false;
                     let mut pathbuf_vec: Vec<PathBuf> = vec![];
                     if metadata(dir_pathbuf.clone()).unwrap().is_dir() {
                         loop {
                             if dir_pathbuf != PathBuf::from("") {
                                 if dir_pathbuf != PathBuf::from("/") {
                                     pathbuf_vec.push(dir_pathbuf.clone());
                                 } else {
                                     break
                                 }
                             } else {
                                 break
                             }
                             dir_pathbuf.pop();
                         }
                         for dir_pathbuf in pathbuf_vec.iter().rev() {
                             let list = self.list.clone().update(dir_pathbuf);
                             self.update(list);
                         }
                         self.halt = false;
                         let halt = self.list.filter.is_some();
                         self.update_file_display(is_fuzzed, halt);
                     }
                     path_cache.switch_back();
                 },
                 "zsh" => {
                     let mut path_cache = command_assistors::PathCache::new(
                         self.list.parent_path.as_path()
                     );
                     path_cache.switch();
                     //Split cmd ('zsh')
                     //let split: Vec<&str> = input.as_read.split("zsh").collect();
                     //let cmd = split.iter().last().unwrap();
                     //let cmd = format!(r#"zsh {}"#, cmd);
                     terminal::shell::spawn("zsh".to_string(), vec![]);
                     path_cache.switch_back();
                 },
                 _ => {
                     let mut path_cache = command_assistors::PathCache::new(
                         self.list.parent_path.as_path()
                     );
                     path_cache.switch();
                     let _output = terminal::shell::cmd(as_read.to_string()).unwrap();
                     path_cache.switch_back();
                 }
             }
        }
    }

    fn key_related_mode(&mut self, input: Result<Option<String>, std::io::Error>, is_fuzzed: bool) {
        match input {
            Ok(t) =>  {
                if let Some(i) = t {
                    let input = Input::new();
                    let input = input.parse(i);
                    // Safe to unwrap.
                    //
                    match input.clone().cmd_type.unwrap() {
                        CmdType::SingleKey => {
                            self.key_mode(self.list.clone(), input, is_fuzzed);
                        },
                        CmdType::MultipleKeys => {
                            /*
                                * get_file_by_key for each key
                                * let text_vec = vec![r#"printf '1=file1; 2=file2;...'; \n "#]
                                * then type_text_spawn(text_vec);
                            */
                            self.return_file_by_key_mode(input, is_fuzzed);
                        },
                        CmdType::FilterKeys => {
                            self.filter_mode(self.list.clone());
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
           let (input,_execute) = self.read_process_chars();
            execute = _execute;
           if execute && !self.input.full_backspace {
               self.key_related_mode(Ok(input), self.is_fuzzed);
           } else if !execute && self.input.full_backspace {
           } else {
               break
           }
           self.input.full_backspace = false;
        }
    }

    fn test_data_update(&mut self, input: Option<String>) {
        if self.test == true {
            if input.is_some() {
                let hash = sha256(&input.clone().unwrap());
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
                let hash = sha256(&self.display.clone().unwrap().1);
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

    fn read_process_chars(&mut self) -> (Option<String>, bool) {
        self.input = Input::new();
        let stdin = stdin();
        let stdout = stdout();
        let stdout = stdout.lock().into_raw_mode().unwrap();
        let mut screen: AlternateScreen<RawTerminal<StdoutLock>> = AlternateScreen::from(stdout);
        let stdin = stdin.lock();
        let mut result: Option<String> =  None;
        let mut is_fuzzed = false;
        let orig_ls_key = self.clone();

        clear_display(&mut screen);

        let input_string: String = self.input.display.iter().collect();
        self.test_data_update(Some(input_string));
        display_files(self.clone(), b"", &mut screen, (0, 3));

        for c in stdin.keys() {
            self.input.full_backspace;
            clear_display(&mut screen);
            let c = c.unwrap();

            self.input.match_event(c);
            let mut input_string: String = self.input.display.iter().collect();
            let input = self.input.clone();
            let first = input.display.iter().nth(0);
            let input = self.input.clone();
            let last = input.display.iter().last();
            let mut _input = self.input.clone();

            let place = (0, 1);
            if let Some(_) = first {

                if self.input.unwiddle {
                    self.fuzzy_list = self.pre_fuzz_list.clone();
                    if let Some(x) = self.pre_fuzz_list.clone() {
                        self.list = x;
                    }
                    if self.input.full_backspace {
                       *self = orig_ls_key.clone();
                       is_fuzzed = false;
                    }
                }
                self.test_data_update(Some(input_string.clone()));
                display_input(input_string.clone(), &mut screen, place);

                let some_mode = self.mode_parse(input_string.clone());

                if let Some(mode) = some_mode {
                    match mode {
                        Mode::Cmd(_) => {
                             if &last == &Some(&'\n') {
                                 self.cmd_read();

                                 //Clear the command from the lsk console after executing.
                                 input_string = "".to_string();
                                 self.input.display = input_string.chars().collect();
                                 clear_display(&mut screen);
                             }
                        },
                        Mode::Work => {
                             if &last == &Some(&'\n') {
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
                            if !is_fuzzed {
                                self.pre_fuzz_list = Some(self.list.clone());
                            }
                            let _show = self.display.clone();
                            let some_keys = parse_keys(fuzzy_mode_input.as_str());

                            if let Some(keys) = some_keys {
                                input_string = keys;
                                self.input.display = input_string.chars().collect();
                            } else {

                                if self.input.unwiddle {
                                    self.fuzzy_list = self.pre_fuzz_list.clone();
                                    if let Some(x) = self.pre_fuzz_list.clone() {
                                        self.list = x;
                                    }
                                }

                                if self.input.display.iter().last() != Some(&'\n') {
                                    self.fuzzy_update(fuzzy_mode_input);
                                }
                            }

                            is_fuzzed = true;
                        }
                    }
                }
            }

            //if self.input.unwiddle {
            //    self.fuzzy_list = self.pre_fuzz_list.clone();
            //    if let Some(x) = self.pre_fuzz_list.clone() {
            //        self.list = x;
            //    }
            //    if self.input.full_backspace {
            //       *self = orig_ls_key.clone();
            //       is_fuzzed = false;
            //    }
            //}


            if self.input.full_backspace {
               *self = orig_ls_key.clone();
               is_fuzzed = false;
            }
            self.test_data_update(Some(input_string.clone()));
            display_files(self.clone(), b"", &mut screen, (0, 3));

            if self.input.display.iter().last() == Some(&'\n') {
                self.input.display.pop();
                let input_string: String = self.input.display.iter().collect();
                result = Some(input_string);
                self.is_fuzzed = is_fuzzed;
                if self.is_fuzzed {
                }
                break
            }
        }

        write!(screen, "{}", termion::cursor::Show).unwrap();

        (result, self.input.execute)
    }

    pub fn mode_parse(&mut self, mut input: String) -> Option<Mode> {
        let len = input.len();
        let mode = if len >= 2 {
             let mode = input.as_str();
             if mode == "w\n" {
                 Some(Mode::Work)
             } else {
                  let mode: String = input.drain(..2).collect();
                  let mode = mode.as_str();
                  match mode {
                      "f " => Some(Mode::Fuzzy(input.clone())),
                      "c " => Some(Mode::Cmd(input.clone())),
                      _ => None
                  }
             }
        } else {
            None
        };

        mode
    }

    fn cmd_read(&mut self) -> String {
         self.input.display.pop();
         let input_string: String = self.input.display.iter().collect();
         let cmd_mode = self.mode_parse(input_string.clone()).unwrap(); //safe

         match cmd_mode {
             Mode::Cmd(cmd_mode_input) => {
                 let input = Input::new();
                 let input = input.parse(cmd_mode_input);

                 match input.clone().cmd_type.unwrap() {
                     CmdType::Cmd => {
                         self.cmd_mode(input);
                     },
                     _ => {}
                 }
                 //break
             }
             _ => { }
         }

        input_string
    }
}


fn clear_display(screen: &mut AlternateScreen<RawTerminal<StdoutLock>>) {
    write!(
        screen,
        "{}",
        termion::clear::All
    ).unwrap();
    screen.flush().unwrap();
}

fn display_input(input_string: String, screen: &mut AlternateScreen<RawTerminal<StdoutLock>>, position: (u16, u16)) {
    write!(screen,
        "{}{}{}{}", format!("{}", input_string.as_str()
        ),
       termion::clear::AfterCursor,
       termion::cursor::Goto(position.0, position.1 + 1),
       termion::cursor::Hide,
    ).unwrap();
    screen.flush().unwrap();
}

fn display_files(ls_key: LsKey, some_stuff: &[u8], screen: &mut AlternateScreen<RawTerminal<StdoutLock>>, position: (u16, u16)) {
     let show = ls_key.clone().display;
     if let Some(x) = show {
         if x.0 == ls_key.list.parent_path {
              //into_raw_mode requires carriage returns.
              let display = str::replace(x.1.as_str(), "\n", "\n\r");
              write!(
                  screen,
                  "{}{}{}\n", format!("{}", std::str::from_utf8(&some_stuff).unwrap()),
                  termion::cursor::Goto(position.0, position.1),
                  termion::cursor::Hide,

              ).unwrap();
              screen.flush().unwrap();

              write!(screen,
                  "{}{}{}{}", format!("{}", display.as_str()
                  ),
                 termion::clear::AfterCursor,
                 termion::cursor::Goto(position.0, position.1 + 1),
                 termion::cursor::Hide,
              ).unwrap();
              screen.flush().unwrap();

              write!(
                  screen,
                  "{}",
                  termion::cursor::Goto(0, 3),
              ).unwrap();
              screen.flush().unwrap();
         }
     }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CmdType {
    SingleKey,
    MultipleKeys,
    FilterKeys,
    Cmd,
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
    pub full_backspace: bool,
}



impl Input {
    pub fn new() -> Self {
        let mut input: Input = Default::default();
        input.execute = true;
        input.unwiddle = false;
        input.full_backspace = false;

        input
    }

    pub fn match_event(&mut self, c: termion::event::Key) {
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
                Key::Left => println!("←"),
                Key::Right => println!("→"),
                Key::Up => println!("↑"),
                Key::Down => println!("↓"),
                Key::Backspace => {
                    self.unwiddle = true;
                    if self.display.pop().is_some() {
                        let count = self.display.iter().count();
                        if count  == 0 {
                            self.execute = false;
                            self.full_backspace = true;
                        }
                    }

                },
                _ => {}
            }
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
        let is_key = if args == None {
            let key: Result<usize, std::num::ParseIntError> = cmd.clone().unwrap().parse();
            match key {
                Ok(_) => Some(true),
                Err(_) => Some(false)
            }
        } else {
            Some(false)
        };

        let is_filter = {
            let mut res = false;
            let mut input_vec_str: Vec<&str> = input.split("-").collect();

            // Make sure it's more than one item (it's range of values).
            if input_vec_str.iter().count() > 1 {
                res = true;
            }

            if res {
                // Does it end in "-"?
                let last = input_vec_str.iter().last();
                let mut open_range = false;
                if let Some(l) = last {
                    if l == &"" {
                        open_range = true;
                   }
                }

                // Only want to deal with integers if it's an open range.
                if open_range {
                   input_vec_str.pop();
                }

                // Make sure it'sall integers.
                for i in input_vec_str.into_iter() {
                    let key: Result<usize, std::num::ParseIntError> = i.parse();
                    if !key.is_ok() {
                        res = false;
                        break;
                    } else {
                        res = true;
                    }
                }
            } // if res

            res
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
            CmdType::MultipleKeys
        } else if is_filter {
            CmdType::FilterKeys
        } else if let Some(k) = is_key {
            if k {
                CmdType::SingleKey
            } else {
                CmdType::Cmd
            }
        } else if is_filter {
            CmdType::FilterKeys
        } else {
            CmdType::Cmd
        };

            //if cmd_type == CmdType::FilterKeys {
            //    let few_ms = std::time::Duration::from_millis(1000);
            //    std::thread::sleep(few_ms);
            //}

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
            let res: Result<usize, std::num::ParseIntError> = x.parse();
            match res {
                Ok(_) => true,
                Err(_) => false
            }
        };
        let is_all_nums = !input.iter().any(|x| !is_num(x.as_str()));

        is_all_nums
     }

     //fn is_key(&self, input: &Vec<String>) -> bool {
     //   if input.iter().count() == 1 {
     //       let key: Result<usize, std::num::ParseIntError> = input.iter().next().unwrap().parse();
     //       match key {
     //           Ok(_) => true,
     //           Err(_) => false
     //       }
     //   } else {
     //       false
     //   }
     //}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Fuzzy(String),
    Cmd(String),
    Work,
}

fn parse_keys(input: &str) -> Option<String> {
    let x = input;
    let mut y: Vec<&str> = x.split(" ").collect();
    let mut count = y.iter().count();

    let some = if count > 1 {
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
    use std::fs::metadata;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use fixture::{Fixture, command_assistors};
    use termion::terminal_size;
    use super::{Input, LsKey, CmdType, Mode};
    use super::*;

    macro_rules! test {
        (
            $test_mode_bool: expr, // If false, tests are screen-size sensitive.
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


                if !$test_mode_bool {
                    let term_size = terminal_size().unwrap();
                    let term_width = term_size.0;
                    let term_height = term_size.1;

                    assert_eq!(term_width, 31);
                    assert_eq!(term_height, 15);
                }

                let spawn = super::terminal::parent_shell::type_text_spawn(text_vec, $delay);
                let fzf = PathBuf::from("/home/me/.fzf.sh");
                let fzc = PathBuf::from("/home/me/.fzc.sh");
                let fzd = PathBuf::from("/home/me/.fzd.sh");
                let _ls_key = super::app::run(test_path_string.clone(), $list_all_bool, true, Some(fzf), Some(fzc), Some(fzd));
                spawn.join().expect("failed to spawn thread");

                let mut test_output_path = path_path.clone();
                test_output_path.push("sample_files");
                test_output_path.push(".lsk_test_output");
                let test_output_path_string = test_output_path.clone().into_os_string().into_string().unwrap();
                let output_mv_to_path_string = path_path.clone().into_os_string().into_string().unwrap();

                Command::new("mv")
                    .arg(test_output_path_string)
                    .arg(output_mv_to_path_string.clone())
                    .output()
                    .expect("failed to execute lsk process");

                let few_ms = std::time::Duration::from_millis(100);
                std::thread::sleep(few_ms);

                let mut output_mv_to_path = path_path.clone();
                output_mv_to_path.push(".lsk_test_output");
                let output_mv_to_path_string = output_mv_to_path.clone().into_os_string().into_string().unwrap();

                println!("\npath:\n{}", output_mv_to_path_string.clone());

                let file256 = file_sha256(output_mv_to_path_string.clone());

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
          true, // test_mode_bool
          false, //list_all_bool
          macro_fzc_enter_file,
          "Makefile",
          100,               //$delay in milleseconds
          "$(printf 'c fzc\r')", //$input1
          "$(printf 'vimread\r')",//$input2
          "$(printf ':q\r')", //$input3
          "$(printf 'q\r')", //$input4
          "",                //$input5
          "",                //$input6
          "",                //$input7
          "macro_fzc_enter_file",
          ">Run lsk\n>Open file using vim with fzc hook\n>Quite vim\n>Quite lsk",
          "64e1b450ebdc532da6ebd5b11df4347d67b48405aa166b05800180e1a1136bf2",
          ignore/*macro_use*/
    );

    test!(
          false, // test_mode_bool
          false, //list_all_bool
          macro_term_size_enter_file,
          "Makefile",
          100,               //$delay in milleseconds
          "$(printf '5\r')", //$input1
          "$(printf ':q\r')",//$input2
          "$(printf 'q\r')", //$input3
          "",                //$input4
          "",                //$input5
          "",                //$input6
          "",                //$input7
          "macro_enter_file",
          ">Run lsk\n>Open file by key (2)\n>Quite vim\n>Quite lsk",
          "6e72cf41634635762cc35c32641d59ffb3eeb449f72dcc218b5cdf7016b7c279",
          ignore/*host_term_size_dependent*/
    );

    test!(
          true, // test_mode_bool
          false, //list_all_bool
          macro_enter_file,
          "Makefile",
          100,               //$delay in milleseconds
          "$(printf '5\r')", //$input1
          "$(printf ':q\r')",//$input2
          "$(printf 'q\r')", //$input3
          "",                //$input4
          "",                //$input5
          "",                //$input6
          "",                //$input7
          "macro_enter_file",
          ">Run lsk\n>Open file by key (2)\n>Quite vim\n>Quite lsk",
          "99150c8a4c5960ee34cfbbb8393a65a9fe1c544394c1d02bf6a0a5cf0ad9b6a9",
          ignore/*macro_use*/
    );

    test!(
          true, // test_mode_bool
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
          "e539e2c5a37d677c59fd71ca1739ae398ed467fc9dd506ec2512533f5d070ae4",
          ignore/*macro_use*/
    );

    test!(
          true, // test_mode_bool
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
          "0ba01081b893ec3fd0e7dcff4577c565c058ce7d8d49d0f601fe7a05dd1e9005",
          ignore/*macro_use*/
    );

    test!(
          true, // test_mode_bool
          false,
          macro_fuzzy_enter_dir,
          "a-file",
          100,               //inrease 200 => 500 ms to see better.
          "$(printf 'f ins\r')",
          "$(printf '4\r')",
          "$(printf 'q\r')",
          "",
          "",
          "",
          "",
          "macro_fuzzy_enter_dir",
          ">Run lsk\n>Fuzzy widdle\n>Open dir by key (1)\n>Quite vim\n>Quite lsk",
          "ffd4c27b3132750622f8c2075064800bec6ae5d8a3654391a1a2aa548bd4a002",
          ignore/*macro_use*/
    );

    test!(
          true, // test_mode_bool
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
          "3438acd815476b946b6196934d31af5b96a240949bd15a2a6d9e3f0b77b505f3",
          ignore/*macro_use*/
    );

    test!(
          true, // test_mode_bool
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
          "2541172a470e8c8ea31f451292eec20999a2a788d60fc0dc3465f743e37b2bed",
          ignore/*macro_use*/
    );

    test!(
          true, // test_mode_bool
          false,
          macro_walk_in_park,
          "a-file",
          100,               //inrease 200 => 500 ms to see better.
          "$(printf '24\r')",
          "$(printf '1\r')",
          "$(printf 'f con\r')",
          "$(printf '2\r')",
          "$(printf ':q\r')",
          "$(printf 'q\r')",
          "",
          "macro_walk_in_park",
          ">Run lsk\n>Go back (0)\n>Fuzzy widdle\n>Open back into original dir by key (2)\n>\n>Quite lsk",
          "1642667a77cde39adce4927ec6a6c61a6773c9d2261b1a4d4fe1d02c7e53cbf0",
          ignore/*macro_use*/
    );

     test!(
           true, // test_mode_bool
           false, //list_all_bool
           macro_fuzzy_backspace,
           "Makefile",
           100,
           "f itf",
           "BackSpace",
           "BackSpace",
           "BackSpace",
           "BackSpace",
           "BackSpace",
           "q\r",
           "macro_fuzzy_backspace",
           ">Run lsk\n>OFuzzy widdle (2)\n>Backspace fully (bad behavior)\n>Quite lsk",
           "cef9ca29708d0077c26750ac76a443cc02c637a9d9bf49dd3bdc990f1fcf4447",
           ignore/*macro_use*/
     );

     test!(
           true, // test_mode_bool
           false, //list_all_bool
           macro_bad_fuzzy_backspace_enter,
           "Makefile",
           100,
           "f itf",
           "BackSpace",
           "BackSpace",
           "",
           "",
           "\r",
           "q\r",
           "macro_bad_fuzzy_backspace_enter",
           ">Run lsk\n>OFuzzy widdle (2)\n>Backspace partially (bad behavior)\n>Quite lsk",
           "cbce709ccb1d782baa4ff20f80076d5f315c683b108e4ecd05a4ba51bd872570",
           ignore/*macro_use*/
     );

     test!(
           true, // test_mode_bool
           false, //list_all_bool
           macro_file_range,
           "Makefile",
           100,
           "20-25\r",
           "24\r",
           "1\r",
           "11\r",
           "",
           ":q\r",
           "q\r",
           "macro_file_range",
           ">Run lsk\n>List range 20-25\n>Enter rust dir\nEnter redox dir\n>Open filesystem.toml\n>Quite Vim\n>Quite lsk",
           "f5f1e7f641b5f348080ca2f86c0dffa8530cfab308cd9ec61d4cb9b8fa4cf3b7",
           ignore/*macro_use*/
     );

     test!(
           true, // test_mode_bool
           true, //list_all_bool
           macro_list_all_fuzzy_file_range,
           "Makefile",
           100,
           "f m\r",
           "1-10\r",
           "10\r",
           "9\r",
           ":q\r",
           "0\r",
           "q\r",
           "macro_list_all_all_file_range",
           ">Run lsk\n>List all\n>Fuzzy search 'm'\n>List range 1-10\n>Enter mk dir\n>Open qemu.mk\n>Quite Vim\n>Go back/up a dir level\n>Quite lsk",
           "c3e7bfb6e9b3051368bca98785942b60a120cd14096b0d38e5f63e1d7be1974d",
           ignore/*macro_use*/
     );

     test!(
           true, // test_mode_bool
           true, //list_all_bool
           macro_list_all_fuzzy_undo_open_range,
           "Makefile",
           100,
           "f i\r",
           "5-17\n",
           "7-\r",
           "17\r",
           ":q\r",
           "1-\n",
           "q\r",
           "macro_list_all_fuzzy_undo_open_range",
           ">Run lsk\n>List all\n>Fuzzy search 'i'\n>List range 5 - 17.\n>List range 7 open-ended\n>Open last one, key 17\n>Quite Vim\n>List entire range, 1-\n>Quite lsk",
           "a4884dc18df5b571708e292e90a9a26b9de11294a0215945c850804a9413900d",
           ignore/*macro_use*/
     );

     test!(
           true, // test_mode_bool
           true, //list_all_bool
           macro_list_all_fuzzy_dir,
           "Makefile",
           100,
           "f i\r",
           "c fzd\r",
           "redoxgitl\r",
           "1\r",
           "1\r",
           ":q\r",
           "q\r",
           "macro_list_all_fuzzy_dir",
           ">Run lsk\n>List all\n>Fuzzy search 'i'\n>List range 5 - 17.\n>List range 7 open-ended\n>Open rust-toolchain fie  with command vim\n>Quite Vim\n>List entire range, 1-\n>Quite lsk",
           "8e79d5a01e252cbf5057b40316bcc71eaf8b6fa9c0b423e92c73881b3af2b5e6",
           ignore/*macro_use*/
     );

     // It's good if this test is broken. For some reason, and bin/<file> doesn't show up when
     // running fzd.
     test!(
           true, // test_mode_bool
           true, //list_all_bool
           macro_bad_list_all_fuzzy_dir,
           "Makefile",
           100,
           "f i\r",
           "7-\n",
           "c fzd\r",
           "bin\r",
           "1\r",
           ":q\r",
           "q\r",
           "macro_list_all_fuzzy_dir",
           ">Run lsk\n>List all\n>Fuzzy search 'i'\n>List range 5 - 17.\n>List range 7 open-ended\n>Open bind dir fzd command, but main.rs doesn't show.\n>Quite lsk",
           "f0314455ac7d9bee466366f83966280ad1292909fe8071dce0af9cb5ff0f40a1",
           ignore/*macro_use*/
     );


    #[test]
    #[ignore]//docker
    fn parse() {
        let input = Input::new();
        let input = input.parse("vim Cargo.toml".to_string());

        assert_eq!(
           Some(CmdType::Cmd),
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
           Some(CmdType::Cmd),
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
           Some(CmdType::Cmd),
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
           Some(CmdType::SingleKey),
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
           Some(CmdType::Cmd),
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
    //fn shell_spawn_vim() {
    //    super::terminal::shell::spawn("vim".to_string(), vec!["-c".to_string(), "vsplit README.md".to_string(), "dev.sh".to_string()]);
    //}

    //#[test]
    //fn shell_pipe_cmd() {
    //    super::terminal::shell::cmd(r#"du -ah . | sort -hr | head -n 10"#.to_string());
    //}

    //#[test]
    //fn shell_cat_cmd() {
    //    super::terminal::shell::cmd("cat Cargo.toml".to_string());
    //}

    //#[test]
    //fn shell_cat() {
    //    super::terminal::shell::spawn("cat".to_string(), vec!["Cargo.toml".to_string()]);
    //}

     #[test]
     #[ignore]//docker
     fn test_mode_parse() {
        let mut ls_key = LsKey::new("/tmp", false, false, None, None, None);
        let input_single = "f something".to_string();
        let some_fuzzy_search_single = ls_key.mode_parse(input_single.clone());

        let input_multi = "f something and more".to_string();
        let some_fuzzy_search_multi = ls_key.mode_parse(input_multi.clone());

        let input_invalid = "fd".to_string();
        let some_fuzzy_search_invalid = ls_key.mode_parse(input_invalid.clone());

        let input_lack_more = "f".to_string();
        let some_fuzzy_search_lack_more = ls_key.mode_parse(input_lack_more.clone());

        let input_wrong = "d something".to_string();
        let some_fuzzy_search_wrong = ls_key.mode_parse(input_wrong.clone());

        assert_eq!(
            some_fuzzy_search_invalid,
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

     #[test]
     #[ignore]//docker
     fn test_bad_mode_parse() {
        let mut ls_key = LsKey::new("/tmp", false, false, None, None, None);

        let input_lack = "f ".to_string();
        let some_fuzzy_search_lack = ls_key.mode_parse(input_lack.clone());

        assert_eq!(
            some_fuzzy_search_lack,
            None
        );
     }
}
