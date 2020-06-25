use std::path::{Path, PathBuf};
use std::fs::metadata;
use std::borrow::Cow;
use walkdir::{DirEntry, WalkDir, Error as WalkDirError};
use ansi_term::{Colour, Style};
use ansi_term::Colour::*;

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    File,
    Dir,
}


#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub path: PathBuf,
    pub file_type: FileType,
    pub key: Option<usize>
}

// Can't alphabetyize PathBuf case insensitively, so we convert to String then back again.
pub fn alphabetize_paths_vec(paths_vec: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut strings_vec: Vec<String> = vec![];
    let mut alphabetized_paths_vec: Vec<PathBuf> = {
        let mut _alphabetized_paths_vec: Vec<PathBuf> = vec![];
        for path in paths_vec.iter() {
            let path = path.clone();
            let path_string = path.into_os_string().into_string().unwrap();
            strings_vec.push(path_string);
        }

        strings_vec.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        for string in strings_vec.iter() {
            _alphabetized_paths_vec.push(PathBuf::from(string));
        }
        _alphabetized_paths_vec
    };

    alphabetized_paths_vec
}

pub fn alphabetize_entry(a: &Entry, b: &Entry) -> std::cmp::Ordering {
    let paths_vec: Vec<PathBuf> = vec![a.path.clone(), b.path.clone()];
    let mut paths_vec = alphabetize_paths_vec(paths_vec.clone());

    if &a.path == &b.path {
        std::cmp::Ordering::Equal
    } else if paths_vec.iter().nth(0).unwrap() == &b.path {
        std::cmp::Ordering::Greater
    } else {
         std::cmp::Ordering::Less
    }
}


#[cfg(test)]
mod test_entries_sort {
    use super::*;
    #[test]
    fn sort_entries() {
        let a = Entry {
            path: PathBuf::from("/a"),
            file_type: FileType::File,
            key: None,
        };

        let b = Entry {
            path: PathBuf::from("/B"),
            file_type: FileType::File,
            key: None,
        };


        let c = Entry {
            path: PathBuf::from("/c"),
            file_type: FileType::File,
            key: None,
        };

        let mut entries = vec![b.clone(), a.clone(), c.clone()];

        entries.sort_by(|a, b| alphabetize_entry(a, b));

        assert_eq!(
            entries,
            vec![a, b, c]
        )
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct List {
    pub files: Vec<Entry>,
    pub parent_path: PathBuf,
    pub path_history: Vec<PathBuf>,
    pub filter: Option<Vec<usize>>
}

impl List {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let mut list: List = Default::default();
        list.parent_path = path.as_ref().to_path_buf();
        list.path_history.push(list.parent_path.clone());

        list
    }

    // Update due to going into a new directory.
    pub fn update<P: AsRef<Path>>(mut self, path: P) -> Self {
        let old_path_history = self.path_history;
        let old_parent_path = self.parent_path;
        let p = path.as_ref().to_str().unwrap();
        let np: String = basename(p, '/').into_owned();
        let basename = Path::new(&np);
        let mut list: List = Default::default();
        self = list;
        self.path_history = old_path_history;
        self.parent_path = old_parent_path.join(basename);
        self.path_history.push(self.parent_path.clone());

        self
    }

    pub fn list_skip_hidden(mut self) -> Result<(Self), std::io::Error> {
        let mut list: List = Default::default();
        let walker = WalkDir::new(&self.parent_path).max_depth(1).into_iter();
        for entry in walker.filter_entry(|e| !list.clone().skip(e)) {
                self = list_maker(entry, self)?;
        }

        Ok(self)
    }

    pub fn list_include_hidden(mut self) -> Result<(Self), std::io::Error> {
        let mut _list: List = Default::default();
        for entry in WalkDir::new(&self.parent_path).max_depth(1) {
                self = list_maker(entry, self)?;
        }

        Ok(self)
    }

    fn replace_shortest_path(mut self, pathbuf: PathBuf) -> Self {
        let path = pathbuf.into_boxed_path();
        let depth_from_root_dir = path.iter().count();

        let parent_dir_count = self.parent_path.iter().count();
        if depth_from_root_dir < parent_dir_count {
            self.parent_path = path.to_path_buf();
        }

        self
    }

    pub fn order_and_sort_list(self, sort: bool, filter: bool, filter_vec: Option<Vec<usize>>) -> Vec<Entry> {
        let mut all_files = self.files.clone();
        let previous_path = self.path_history.iter().last().unwrap();
        if sort {
            all_files.sort_by(|a, b| alphabetize_entry(a, b));
        }
        all_files.insert(
            0,
            Entry {
                path: previous_path.to_path_buf(),
                file_type: FileType::Dir,
                key: None
            }
        );

        let count = all_files.iter().count();

        // Add keys
        let mut n = 0;
        let mut final_all_files: Vec<Entry> = vec![];
        for mut x in all_files.into_iter() {
            x.key = Some(n);
            final_all_files.push(x.clone());
            n += 1;
        }

        //let few_ms = std::time::Duration::from_millis(1000);
        //std::thread::sleep(few_ms);

        // Filter entries
        if filter {


            let entry_test = Entry {
                path: PathBuf::from("entry_test"),
                file_type: FileType::File,
                key: Some(3)

            };

           fn filter_closure(x: &Entry, filter: &Option<Vec<usize>>) -> bool {
               if let Some(fl) = filter {
                   if let Some(key) = x.key {
                       !fl.iter().find(|n| &key == *n).is_some()
                   } else {
                       false
                   }
               } else {
                   false
               }
           }

           let find_in_range = final_all_files.clone().into_iter().filter(|x|
               // !filter.as_ref().unwrap().iter().find(|n| &x.key.unwrap() == *n).is_some()
               filter_closure(x, &filter_vec)
           );

            let files: Vec<Entry> = find_in_range.collect();

            let entry_test = Entry {
                path: PathBuf::from("entry_test"),
                file_type: FileType::File,
                key: Some(3)

            };

            if files != vec![entry_test] {
                //let few_ms = std::time::Duration::from_millis(1000);
                //std::thread::sleep(few_ms);
            } else {
                //let few_ms = std::time::Duration::from_millis(1000);
                //std::thread::sleep(few_ms);
            }
            final_all_files = files;
        } else {
        }

        final_all_files
    }

    pub fn get_file_by_key(&self, key: usize, sort: bool) -> Option<PathBuf> {
        let all_files = order_and_sort_list(&self, sort);
        let all_files = all_files.iter();

        for entry in all_files.clone() {
            //println!("{} [{}]", entry.display(), n);
            //p
            let n = entry.key.unwrap();
            //let parent_file_name = file_or_dir_name(&self.parent_path);
            if n == key {
                let path = entry.path.to_path_buf();
                return self.clone().full_entry_path(path);
            }
        }

        return None
    }

    // Caution ahead, side-effect: set parent dir field.
    fn skip(mut self, entry: &DirEntry) -> bool {

        //if is_parent_dir(entry) {
        //    return true
        //}
        entry.file_name()
             .to_str()
             .map(|s| s.starts_with("."))
             .unwrap_or(false)
    }

    fn full_entry_path(self, path: PathBuf) -> Option<PathBuf> {
        let p = self.parent_path;
        Some(p.join(path.as_path()))
    }
}

fn basename<'a>(path: &'a str, sep: char) -> Cow<'a, str> {
    let mut pieces = path.rsplit(sep);
    match pieces.next() {
        Some(p) => p.into(),
        None => path.into(),
    }
}

fn file_or_dir_name(path: &PathBuf) -> Option<PathBuf> {
    let path = path.as_path();
    let path = path.file_name();

    if let Some(p) = path {
        Some(Path::new(&p).to_path_buf())
    } else {
        None
    }
}

fn list_maker(entry: Result<(DirEntry), WalkDirError>, mut list: List) -> Result<(List), std::io::Error> {
    match entry {
        Ok(entry) => {
            let entry = entry.path();
            let previous_path = list.path_history.iter().last().unwrap();
            let parent_file_name = file_or_dir_name(&list.parent_path);

                match metadata(entry) {
                    Ok(md) => {
                       let path = entry.to_path_buf();
                       let short_path = file_or_dir_name(&path);
                       if md.is_file() {
                           list = list.replace_shortest_path(path);
                           if let Some(p) = short_path {

                               if Some(p.clone()) != parent_file_name {
                                   list.files.push(
                                       Entry {
                                           path: p,
                                           file_type: FileType::File,
                                           key: None
                                       }
                                    );
                               }
                           }
                       } else if md.is_dir() {
                           list = list.replace_shortest_path(path);
                           if let Some(p) = short_path {
                               if Some(p.clone()) != parent_file_name {
                                   list.files.push(
                                       Entry {
                                           path: p,
                                           file_type: FileType::Dir,
                                           key: None
                                       }
                                    );
                               }
                           }
                       }
                    },
                    Err(_) => ()
                }
        },
        Err(_) => ()
    }

    Ok(list)
}

pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
    metadata(path).unwrap().is_file()
}

pub fn is_dir<P: AsRef<Path>>(path: P) -> bool {
    metadata(path).unwrap().is_dir()
}

//pub fn go_back_compoenent_display() {
//    let _previous_path = previous_path.as_path();
//    let components = _previous_path.components();
//    let component = components.last().unwrap();
//    let last = component.as_os_str();
//}

pub fn key_entries(entries: Vec<Entry>, filter: Option<Vec<usize>>) -> Vec<String> {
    let mut entries_keyed: Vec<String> = vec![];
    for entry in entries.clone() {
        let n = entry.key.unwrap();
        let entry = match entry.file_type {
            FileType::File => {
                let entry = entry.path.to_str().unwrap();
                let entry = format!(r#"{} [{}]"#, entry, n);
                Colour::White.bold().paint(entry).to_string()
            },
            FileType::Dir => {
                if n == 0 {
                    let _path = entry.path.clone();
                    let _path = _path.as_path();
                    let os_str = _path.iter().last().unwrap();
                    let mut entry_str = os_str.to_str().unwrap();
                    let mut entry_string = entry_str.to_string();
                    if entry_str != "/" {
                          entry_string = format!("../{}", entry_str);
                          Colour::Blue.bold().paint(entry_string).to_string()
                    } else {
                        "/".to_string()
                    }
                } else {
                    let entry_str = entry.path.to_str().unwrap();
                    let entry = format!(r#"{} [{}]"#, entry_str, n);
                    Colour::Blue.bold().paint(entry).to_string()
                }
            },
        };
        if entry != "/".to_string() {
             entries_keyed.push(entry);
        }
    }

    entries_keyed
}

pub fn order_and_sort_list(list: &List, sort: bool) -> Vec<Entry> {
    let mut all_files = list.files.clone();
    let previous_path = list.path_history.iter().last().unwrap();
    if sort {
        all_files.sort_by(|a, b| alphabetize_entry(a, b));
        //all_files = alphabetize_paths_vec(all_files.clone());
    }
    all_files.insert(
        0,
        Entry {
            path: previous_path.to_path_buf(),
            file_type: FileType::Dir,
            key: None
        }
    );

    let count = all_files.iter().count();

    //all_files.iter().map(|mut x|
    //    (0..count).for_each(|n| all_files.into_iter().nth(n).unwrap().key = Some(n))
    //);
    //

    let mut n = 0;
    let mut final_all_files: Vec<Entry> = vec![];
    for mut x in all_files.into_iter() {
        x.key = Some(n);
        final_all_files.push(x.clone());
        n += 1;
    }

    //(0..count).for_each(|n| (all_files.iter().nth(n).unwrap() = Some(n)));

    final_all_files
}

pub fn print_list_with_keys(list: List) -> Result<(), std::io::Error> {
    let all_files = order_and_sort_list(&list, true);
    let mut n = 0;
    for entry in all_files {
        println!("{} [{}]", entry.path.display(), n);
        n += 1;
    }

    Ok(())
}

pub mod fuzzy_score {
    use super::{FileType, Entry};
    use fuzzy_matcher;
    use std::path::PathBuf;
    use fuzzy_matcher::FuzzyMatcher;
    use fuzzy_matcher::skim::SkimMatcherV2;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Score {
         Files((Entry, Option<(i64, Vec<usize>)>)),
    }

    impl Score {
        pub fn score(&self) -> (Entry, Option<(i64, Vec<usize>)>) {
            match self.clone() {
                Score::Files(score) => score,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Scores {
        pub files: Vec<Score>,
    }

    pub fn score(compare_to: &str, guess: &str) -> Option<(i64, Vec<usize>)> {
        let matcher = SkimMatcherV2::default();
        matcher.fuzzy_indices(compare_to, guess)
    }

    pub fn order(a: &Score, b: &Score) -> std::cmp::Ordering {
        let a_score = a.score().0.path;
        let b_score = b.score().0.path;

        if a_score == b_score {
            std::cmp::Ordering::Equal
        } else if a_score > b_score {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }
}

#[cfg(test)]
mod fuzzy_tests {
    use std::path::PathBuf;
    use super::*;
    #[test]
    #[ignore]//docker
    fn score() {
        let res = super::fuzzy_score::score("abc", "abx");
        assert_eq!(res, None);
        let (score, indices) = super::fuzzy_score::score("axbycz", "xyz").unwrap();
        assert_eq!(indices, [1, 3, 5]);
        assert_eq!(score, 39);
        let (score, indices) = super::fuzzy_score::score("axbycz", "abc").unwrap();
        assert_eq!(indices, [0, 2, 4]);
        assert_eq!(score, 55);
        let (score, indices) = super::fuzzy_score::score("unignore_play_test", "uigp").unwrap();
        assert_eq!(indices, [0, 2, 3, 9]);
        assert_eq!(score, 78);
    }

    #[test]
    #[ignore]//docker
    fn sort_score() {
        let guess = "xyz";
        let file_a = "xayb";
        let file_b = "xyazabc";
        let file_c = "xyza";
        let file_d = "afd";
        let dir_a = "dirxyzabc";
        let dir_b = "dirxzabc";

        let res_a = super::fuzzy_score::score(file_a, guess);
        let res_b = super::fuzzy_score::score(file_b, guess);
        let res_c = super::fuzzy_score::score(file_c, guess);
        let res_d = super::fuzzy_score::score(file_d, guess);

        let mut scores = super::fuzzy_score::Scores {
            files: vec![
                     super::fuzzy_score::Score::Files(
                         (
                             Entry {
                                 path: PathBuf::from(file_a),
                                 file_type: FileType::File,
                                 key: None
                             },
                             res_a.clone()
                         )
                     ),
                     super::fuzzy_score::Score::Files(
                         (
                             Entry {
                                 path: PathBuf::from(file_b),
                                 file_type: FileType::File,
                                 key: None
                             },
                             res_b.clone()
                         )
                     ),
                     super::fuzzy_score::Score::Files(
                         (
                             Entry {
                                 path: PathBuf::from(file_c),
                                 file_type: FileType::File,
                                 key: None
                             },
                             res_c.clone()
                         )
                     ),
                     super::fuzzy_score::Score::Files(
                         (
                             Entry {
                                 path: PathBuf::from(file_d),
                                 file_type: FileType::File,
                                 key: None
                             },
                             res_d.clone()
                         )
                     ),
                 ],
        };

        let pre_sort = scores.clone();
        scores.files.sort_by(|a, b| fuzzy_score::order(a, b));
        let post_sort = scores.clone();
        assert_ne!(
            pre_sort,
            post_sort
        );
        //scores.dirs.sort_by(|a, b| fuzzy_score::order(a, b));

        assert_ne!(
            pre_sort.files,
            post_sort.files
        );

        assert_eq!(
             scores.files.iter().count(),
             4
        );

        assert_eq!(
            scores.files.iter().nth(0).unwrap().score().0.path,
            PathBuf::from("xyza")
        );

        assert_eq!(
            scores.files.iter().nth(1).unwrap().score().0.path,
            PathBuf::from("xyazabc")
        );

        assert_eq!(
            scores.files.iter().nth(2).unwrap().score().0.path,
            PathBuf::from("xayb")
        )
    }
}



#[cfg(test)]
mod tests {
    use std::fs::metadata;
    use std::path::{Path, PathBuf};
    use fixture::Fixture;

    #[test]
    #[ignore]//docker
    fn current_print_list_include_hidden() {
        let path = "./";

        let list = super::List::new(path)
            .list_include_hidden()
            .unwrap();

        super::print_list_with_keys(list).unwrap();
    }

    #[test]
    #[ignore]//docker
    fn get_non_hidden_paths_by_key() {
        let path = "/tmp/lsk_tests/get_non_hidden_paths_by_key/";

        let mut fixture = Fixture::new()
            .add_dirpath(path.to_string())
            .add_dirpath(path.to_string() + "a-dir")
            .add_dirpath(path.to_string() + ".a-hidden-dir")
            .add_file(path.to_string() + "a-file")
            .add_file(path.to_string() + "a-dir/a-file")
            .add_file(path.to_string() + "a-dir/b-file")
            .add_file(path.to_string() + ".a-hidden-dir/a-file")
            .add_file(path.to_string() + ".a-hidden-dir/.a-hidden-file")
            .add_file(path.to_string() + ".a-hidden-file")
            .build();

        let list = super::List::new(path)
            .list_skip_hidden()
            .unwrap();

        let file_path_1 = list.get_file_by_key(1, true).unwrap();
        let file_path_2 = list.get_file_by_key(2, true).unwrap();
        let file_path_3 = list.get_file_by_key(3, true);

        assert_eq!(
            true,
            metadata(file_path_1.clone()).unwrap().is_dir()
        );
        assert_eq!(
            true,
            metadata(file_path_2.clone()).unwrap().is_file()
        );

        fixture.teardown(true);

        assert_eq!(file_path_1, Path::new("/tmp/lsk_tests/get_non_hidden_paths_by_key/a-dir").to_path_buf());
        assert_eq!(file_path_2, Path::new("/tmp/lsk_tests/get_non_hidden_paths_by_key/a-file").to_path_buf());
        assert_eq!(file_path_3, None);
    }

    #[test]
    #[ignore]//docker
    fn get_all_paths_by_key() {
        let path = "/tmp/lsk_tests/get_all_paths_by_key/";

        let mut fixture = Fixture::new()
            .add_dirpath(path.to_string())
            .add_dirpath(path.to_string() + "a-dir")
            .add_dirpath(path.to_string() + ".a-hidden-dir")
            .add_file(path.to_string() + "a-file")
            .add_file(path.to_string() + "a-dir/a-file")
            .add_file(path.to_string() + "a-dir/b-file")
            .add_file(path.to_string() + ".a-hidden-dir/a-file")
            .add_file(path.to_string() + ".a-hidden-dir/.a-hidden-file")
            .add_file(path.to_string() + ".a-hidden-file")
            .build();

        let list = super::List::new(path)
            .list_include_hidden()
            .unwrap();

        let file_path_1 = list.get_file_by_key(1, true).unwrap();
        let file_path_2 = list.get_file_by_key(2, true).unwrap();
        let file_path_3 = list.get_file_by_key(3, true).unwrap();
        let file_path_4 = list.get_file_by_key(4, true).unwrap();
        let file_path_5 = list.get_file_by_key(5, true);

        //assert_eq!(
        //    true,
        //    metadata(file_path_1.clone()).unwrap().is_file()
        //);
        //assert_eq!(
        //    true,
        //    metadata(file_path_2.clone()).unwrap().is_dir()
        //);

        fixture.teardown(true);

        assert_eq!(file_path_1, Path::new("/tmp/lsk_tests/get_all_paths_by_key/.a-hidden-dir").to_path_buf());
        assert_eq!(file_path_2, Path::new("/tmp/lsk_tests/get_all_paths_by_key/.a-hidden-file").to_path_buf());
        assert_eq!(file_path_3, Path::new("/tmp/lsk_tests/get_all_paths_by_key/a-dir").to_path_buf());
        assert_eq!(file_path_4, Path::new("/tmp/lsk_tests/get_all_paths_by_key/a-file").to_path_buf());
        assert_eq!(file_path_5, None);
    }
}
