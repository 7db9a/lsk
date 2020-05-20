use std::path::{Path, PathBuf};
use std::fs::metadata;
use walkdir::{DirEntry, WalkDir, Error as WalkDirError};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct List {
    pub files: Vec<PathBuf>,
    pub dirs: Vec<PathBuf>,
    pub relative_parent_dir_path: PathBuf,
    pub parent_dir: Option<PathBuf>,
    pub path_history: Vec<PathBuf>
}

impl List {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let mut list: List = Default::default();
        list.relative_parent_dir_path = path.as_ref().to_path_buf();
        list.path_history.push(list.relative_parent_dir_path.clone());
        list
    }

    pub fn update_path_history() {

    }

    pub fn list_skip_hidden(mut self) -> Result<(Self), std::io::Error> {
        let mut _list: List = Default::default();
        let walker = WalkDir::new(self.relative_parent_dir_path.clone()).max_depth(1).into_iter();
        for entry in walker.filter_entry(|e| !_list.clone().skip(e)) {
                self = list_maker(entry, self.clone())?;
        }

        Ok(self)
    }

    pub fn list_include_hidden(mut self) -> Result<(Self), std::io::Error> {
        let mut _list: List = Default::default();
        for entry in WalkDir::new(self.relative_parent_dir_path.clone()).max_depth(1) {
                self = list_maker(entry, self.clone())?;
        }

        Ok(self)
    }

    fn replace_shortest_path(mut self, pathbuf: PathBuf) -> Self {
        let path = pathbuf.into_boxed_path();
        let depth_from_root_dir = path.iter().count();

        if let Some(x) = self.clone().parent_dir {
            let parent_dir_count = x.iter().count();
            if depth_from_root_dir < parent_dir_count {
                self.parent_dir = Some(path.to_path_buf());
            }
        } else {
            self.parent_dir = Some(path.to_path_buf());
        }

        self
    }

    pub fn get_file_by_key(&self, key: usize) -> Option<PathBuf> {
        let all_files = order_and_sort_list(self.clone());
        let all_files = all_files.iter();
        let mut done = false;

        while !done {
            let mut n = 1;
            if all_files.clone().count() > 0 {
                for entry in all_files.clone() {
                    //println!("{} [{}]", entry.display(), n);
                    let path = entry.to_path_buf();
                    let parent = self.relative_parent_dir_path.clone();
                    let parent_file_name = file_or_dir_name(parent);
                    if Some(path.clone()) != parent_file_name {
                        if n == key {
                            return self.clone().full_entry_path(path);
                        }
                       n += 1;
                    }
                }
            }

            done = true;
        }

        return None
    }

    // Caution ahead, side-effect: set parent dir field.
    fn skip(mut self, entry: &DirEntry) -> bool {

        //if is_parent_dir(entry) {
        //    self.parent_dir = entry.path().to_path_buf();
        //    return true
        //}
        entry.file_name()
             .to_str()
             .map(|s| s.starts_with("."))
             .unwrap_or(false)
    }

    fn full_entry_path(self, path: PathBuf) -> Option<PathBuf> {
        if let Some(p) = self.parent_dir.clone() {
            Some(p.join(path.as_path()))
        } else {
            None
        }
    }
}

fn file_or_dir_name(path: PathBuf) -> Option<PathBuf> {
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
        Ok(x) => {
            let entry = x;
            let entry = entry.path();
            let md = metadata(entry);
            match md {
                Ok(md) => {
                   let path = entry.to_path_buf();
                   let short_path = file_or_dir_name(path.clone());
                   if md.is_file() {
                       list = list.replace_shortest_path(path);
                       if let Some(p) = short_path {
                           list.files.push(p);
                       }
                   } else if md.is_dir() {
                       list = list.replace_shortest_path(path);
                       if let Some(p) = short_path {
                           list.dirs.push(p);
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

pub fn key_entries(entries: Vec<PathBuf>) -> Vec<String> {
    let mut n = 1;
    let mut entries_keyed: Vec<String> = vec![];
    for entry in entries.clone() {
        let entry = entry.to_str().unwrap();
        let entry = format!(r#"{} [{}]"#, entry, n);
        entries_keyed.push(entry);
        //println!("{} [{}]", entry.display(), n);
        n += 1;
    }

    entries_keyed
}

pub fn order_and_sort_list(list: List) -> Vec<PathBuf> {
    let files = list.files.iter();
    let dirs = list.dirs.iter();
    let mut done = false;

    let mut all_files: Vec<PathBuf> = vec![];

    while !done {
        if files.clone().count() > 0 {
            for entry in files.clone() {
                //println!("{} [{}]", entry.display(), n);
                all_files.push(entry.to_path_buf());
            }
        }
        if dirs.clone().count() > 0 {
            for entry in dirs.clone() {
                if let Some(x) = list.parent_dir.clone() {
                    let parent_file_name = file_or_dir_name(x);
                    if Some(entry) != parent_file_name.as_ref() {
                       all_files.push(entry.to_path_buf());
                    }
                }
            }
        }
        done = true;
    }

    all_files.sort();

    all_files
}

pub fn print_list_with_keys(list: List) -> Result<(), std::io::Error> {
    let all_files = order_and_sort_list(list);
    let mut n = 1;
    for entry in all_files {
        println!("{} [{}]", entry.display(), n);
        n += 1;
    }

    Ok(())
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
        let path = "/tmp/lsk_tests/";

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

        let file_path_1 = list.get_file_by_key(1).unwrap();
        let file_path_2 = list.get_file_by_key(2).unwrap();
        let file_path_3 = list.get_file_by_key(3);

        assert_eq!(
            true,
            metadata(file_path_1.clone()).unwrap().is_dir()
        );
        assert_eq!(
            true,
            metadata(file_path_2.clone()).unwrap().is_file()
        );

        fixture.teardown(true);

        assert_eq!(file_path_1, Path::new("/tmp/lsk_tests/a-dir").to_path_buf());
        assert_eq!(file_path_2, Path::new("/tmp/lsk_tests/a-file").to_path_buf());
        assert_eq!(file_path_3, None);
    }

    #[test]
    #[ignore]//docker
    fn get_all_paths_by_key() {
        let path = "/tmp/lsk_tests/";

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

        let file_path_1 = list.get_file_by_key(1).unwrap();
        let file_path_2 = list.get_file_by_key(2).unwrap();
        let file_path_3 = list.get_file_by_key(3).unwrap();
        let file_path_4 = list.get_file_by_key(4).unwrap();
        let file_path_5 = list.get_file_by_key(5);

        //assert_eq!(
        //    true,
        //    metadata(file_path_1.clone()).unwrap().is_file()
        //);
        //assert_eq!(
        //    true,
        //    metadata(file_path_2.clone()).unwrap().is_dir()
        //);

        fixture.teardown(true);

        assert_eq!(file_path_1, Path::new("/tmp/lsk_tests/.a-hidden-dir").to_path_buf());
        assert_eq!(file_path_2, Path::new("/tmp/lsk_tests/.a-hidden-file").to_path_buf());
        assert_eq!(file_path_3, Path::new("/tmp/lsk_tests/a-dir").to_path_buf());
        assert_eq!(file_path_4, Path::new("/tmp/lsk_tests/a-file").to_path_buf());
        assert_eq!(file_path_5, None);
    }
}
