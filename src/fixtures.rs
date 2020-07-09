use std::fs::{create_dir_all, remove_dir_all, File};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Fixture {
    pub paths: Vec<String>,
    //dirs: Vec<Path>
    files: Vec<String>,
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
