use git2::*;
use std::fs::canonicalize;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Fixture {
    pub paths: Vec<String>,
    //dirs: Vec<Path>
    files: Vec<String>,
    gits: Vec<String>,
}

/// Create directories, files, and tear them down at will.
/// Used throughout immutag crates for testing purposes.
/// See tests directories of said crates for examples.
impl Fixture {
    pub fn new() -> Fixture {
        let fixture: Fixture = Default::default();

        fixture
    }

    pub fn add_dirpath(&mut self, path: String) -> Fixture {
        self.paths.push(path);

        self.clone()
    }

    pub fn add_file(&mut self, file: String) -> Fixture {
        self.files.push(file);

        self.clone()
    }

    pub fn add_git(&mut self, path: String) -> Fixture {
        self.gits.push(path);

        self.clone()
    }

    /// Builds all the directory files and paths and then writes them to disk.
    pub fn build(&mut self) -> Fixture {
        let mkdirs = |path: String| {
            create_dir_all(Path::new(&path.clone())).expect("Failed to create directories.");
            path
        };

        // Create all dir paths.
        self.paths
            .iter()
            .filter(|path| path != &"")
            .for_each(|path| {
                mkdirs(path.to_string());
            });

        //let data = "version = '0.1.0'\n\n[file]\n\n[dir]\n\n[ignore]";
        //let data = "[dir.tests]\nmeta = 'Tests'";

        let touch = |path: String| {
            File::create(path.clone()).expect("Failed to create file.");
            path
        };

        self.files
            .iter()
            .filter(|path| path != &"")
            .for_each(|path| {
                let _path = Path::new(path);
                touch(path.to_string());
            });

        //
        // Initialize any git repos.
        //
        let git_init = |path: String| {
            Repository::init(&path.clone()).expect("failed to git init");
            path
        };

        self.gits
            .iter()
            .filter(|path| path != &"")
            .for_each(|path| {
                let _path = Path::new(path);
                git_init(path.to_string());
            });

        self.clone()
    }

    /// Teardown everything.
    /// At the moment, it's really only suited for tearing down all the files, except the .immutag dir itself.
    /// However, more fine grained control can be added within this method.
    /// For example, teardown could take a Vec of paths to delete.
    pub fn teardown(
        &mut self,
        del_all: bool, /*dir_index: Option<usize>, file_index: Option<usize> */
    ) -> Fixture {
        let rmr = |path: String| {
            remove_dir_all(path.clone()).expect("Failed to delete all files.");
            path
        };

        // Create all dir paths.
        self.paths
            .iter()
            // The first one is the initialization.
            .filter(|path| path != &"")
            // Kinda weird, but useful code here because we may
            // want to delete specific paths in the future.
            // Current usage is to delete the entire root dir
            // structure that was previously built.
            .for_each(|path| {
                if del_all {
                    // Throws an error if the path doesn't exist, so we only remove
                    // if the path exists.
                    if Path::new(path).exists() {
                        rmr(path.to_string());
                    }
                }
            });

        self.clone()
    }
}

pub fn is_git<T: AsRef<str>>(path: T) -> bool {
    let is_git = match Repository::open(Path::new(path.as_ref())) {
            Ok(_) => true,
            Err(_) => false
    };

    is_git
}


#[derive(PartialEq, Clone)]
pub enum EntryType {
    File,
    Dir,
}

pub trait Entry: Sized {
    fn decon_to_string(&self) -> String;
    fn entry_type(&self) -> EntryType;
}

#[derive(Debug, PartialEq)]
/// Name of the file, which is typically the same name when checking the file into git.
pub struct FileEntry(pub String);

#[derive(Debug, PartialEq)]
/// Path name of the dir relative to git project.
pub struct DirEntry(pub String);

impl Entry for FileEntry {
    fn decon_to_string(&self) -> String {
        let FileEntry(value) = self;
        filefy(value.to_string())
    }

    fn entry_type(&self) -> EntryType {
        EntryType::File
    }
}

impl<'a> Entry for &'a FileEntry {
    fn decon_to_string(&self) -> String {
        let FileEntry(value) = self;
        filefy(value.to_string())
    }

    fn entry_type(&self) -> EntryType {
        EntryType::File
    }
}

impl Entry for DirEntry {
    fn decon_to_string(&self) -> String {
        let DirEntry(value) = self;
        directorate(value.to_string())
    }

    fn entry_type(&self) -> EntryType {
        EntryType::Dir
    }
}

impl<'a> Entry for &'a DirEntry {
    fn decon_to_string(&self) -> String {
        let DirEntry(value) = self;
        directorate(value.to_string())
    }

    fn entry_type(&self) -> EntryType {
        EntryType::Dir
    }
}

/// Forces a stringly typed path to always be in a file convention for Immutag file.
pub fn filefy(name: String) -> String {
    let path_name = Path::new(&name);
    let mut name = windows_to_unix_path_string(&path_name).unwrap();

    let dir_split_res = name.split('/').last();
    if let Some(x) = dir_split_res {
        if x == "" {
            name.pop();
            //println!("filefied: {}", name);
            name
        } else {
            name
            //"Very wrong!".to_string()
        }
    } else {
        name
    }
}

/// Forces a stringly typed path to always be in a dir convention for Immutag file.
pub fn directorate(name: String) -> String {
    let path_name = Path::new(&name);
    let mut name = windows_to_unix_path_string(&path_name).unwrap();

    let dir_split_res = name.split('/').last();
    if let Some(x) = dir_split_res {
        if x == "" {
            name
        } else if x == r#"\"# {
            name.pop();
            name + "/"
        } else {
            name + "/"
            //"Very wrong!".to_string()
        }
    } else {
        name
    }
}

/// Not Enbabled: converts a windows path to unix.
/// Requires:
/// use path_slash::PathExt;
fn windows_to_unix_path_string(path: &Path) -> Option<String> {
    Some(path.to_str().unwrap().to_string())
    //path.to_slash()
}

pub fn current_workdir_full_path() -> PathBuf {
    let path = Path::new("./");
    canonicalize(&path).expect("failed to canonicalize path")
}

/// Switch back and forth between paths when executing test commands.
pub mod command_assistors {
    use std::env;
    use std::path::Path;

    pub struct PathCache<'s> {
        from_path: Box<Path>,
        to_path: &'s Path,
    }

    impl<'s> PathCache<'s> {
        pub fn new(to_path: &Path) -> PathCache {
            let current_dir = env::current_dir().expect("failed to get current dir");
            let from_path = current_dir.into_boxed_path();

            PathCache { from_path, to_path }
        }

        pub fn switch(&mut self) {
            if env::set_current_dir(&self.to_path).is_err() {
                panic!("failed to switch back to original dir")
            }
        }

        pub fn switch_back(&mut self) {
            if env::set_current_dir(&self.from_path).is_err() {
                panic!("failed to switch back to original dir")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filefy_test() {
        let good_file = filefy("/a/file".to_string());
        let bad_file = filefy("a/file/with/bad/form/".to_string());
        let ugly_file = filefy("file/".to_string());
        let a_file = filefy("file".to_string());

        assert_eq!("/a/file", &good_file);
        assert_eq!("a/file/with/bad/form", &bad_file);
        assert_eq!("file", &ugly_file);
        assert_eq!("file", &a_file);
    }

    #[test]
    fn directorcate_test() {
        let current_dir = directorate("".to_string());

        let good_dir = directorate("/a/dir/".to_string());
        let bad_dir = directorate("a/dir/with/bad/form".to_string());
        let ugly_dir = directorate("dir".to_string());

        assert_eq!("", &current_dir);

        assert_eq!("/a/dir/", &good_dir);
        assert_eq!("a/dir/with/bad/form/", &bad_dir);
        assert_eq!("dir/", &ugly_dir);
    }

    #[test]
    fn windows_to_unix_path_string_test() {
        let unix_path = Path::new(r#"windows/path/"#);
        //let windows_path = Path::new(r#"windows\path\"#);

        assert_eq!(
            windows_to_unix_path_string(unix_path),
            Some("windows/path/".to_string())
        )
        //assert_eq!(
        //    windows_to_unix_path_string(windows_path),
        //    Some("windows/path/".to_string())
        //)
    }
}
