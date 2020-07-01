extern crate seahorse;
extern crate ls_key;

use std::path::{Path, PathBuf};
use std::env;
use ls_key::{list, app};
use list::{List, is_dir};
use seahorse::{App, Command, Context, Flag, FlagType};

fn main() {
    let args: Vec<String> = env::args().collect();
    let app = App::new()
        .name(env!("CARGO_PKG_NAME"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage("cli [path]")
        .action(default_action)
        .flag(Flag::new("all", "cli [path] --all(-a)", FlagType::Bool).alias("a"))
        .command(is_dir_command())
        .command(get_file_by_key_command());

    app.run(args);
}

fn get_file_by_key_action(c: &Context) {
    let mut args = c.args.iter();
    let mut path = "";
    let mut key = "";
    let arg_count = args.clone().count();
    match arg_count {
       2 => {
           path = args.next().unwrap();
           key = args.next().unwrap();
       },
       1 => {
           path = "";
           key = args.next().unwrap();
       },
       _ => ()
    };

    let use_path: PathBuf;
    if path != "" {
        use_path = Path::new(path).to_path_buf();
    } else {
        use_path = env::current_dir().unwrap();
    }

    let list = List::new(use_path)
        .list_skip_hidden()
        .unwrap();

    let key: usize = key.parse().expect("The key must parseable into an integer.");
    let res = list.get_file_by_key(key, true);
    if let Some(x) = res {
       println!("{}", x.to_str().unwrap());
    }
}

fn is_dir_action(c: &Context) {
    let mut args = c.args.iter();
    let arg_count = args.clone().count();
    let path = match arg_count {
       1 => args.next().unwrap(),
       0 => "",
       _ => ""
    };

    let use_path: PathBuf;
    if path != "" {
        use_path = Path::new(path).to_path_buf();
    } else {
        use_path = env::current_dir().unwrap();
    }

    let res = is_dir(use_path);

    if res {
       println!("0");
    } else {
       println!("1");
    }
}

fn is_dir_command() -> Command {
    Command::new()
        .name("is-dir")
        .usage("cli is-dir [dir]")
        .action(is_dir_action)
}

fn get_file_by_key_command() -> Command {
    Command::new()
        .name("get-file")
        .usage("cli get-file [dir] [key]")
        .action(get_file_by_key_action)
}

fn default_action(c: &Context) {
    let mut args = c.args.iter();
    let arg_count = args.clone().count();
    let path = match arg_count {
       1 => args.next().unwrap(),
       0 => "",
       _ => ""
    };

    if c.bool_flag("all") {
        if path != "" {
            app::run(path, true, false);
        } else {
            app::run(env::current_dir().unwrap(), true, false);
        }
    } else {
        if path != "" {
            app::run(path, false, false);
        } else {
            app::run(env::current_dir().unwrap(), false, false);
        }
    }
}

#[cfg(test)]
mod cli {
    use std::fs::metadata;
    use std::path::Path;
    use fixture::Fixture;
    use fixture::command_assistors;
    use std::process::Command;

    #[test]
    #[ignore]//docker
    fn print_list_include_hidden() {
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

        let path_path = Path::new(path).to_path_buf();
        let mut path_cache = command_assistors::PathCache::new(&path_path);

        // Changing directories.
        path_cache.switch();
        let output = Command::new("/ls-key/target/debug/lsk")
            .arg("-a")
            .output()
            .expect("failed to execute lsk process");

        //assert_eq!(
        //    String::from_utf8_lossy(&output.stdout),
        //    "Initialized immutag in the current directory.\n"
        //);
        //
        //println!("results:\n {}", String::from_utf8_lossy(&output.stdout));

        path_cache.switch_back();

        assert_eq!(true, metadata(path.to_string() + "a-dir").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + ".a-hidden-dir").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "a-file" ).unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + "a-dir/a-file").unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + "a-dir/b-file").unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + ".a-hidden-dir/a-file").unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + ".a-hidden-dir/.a-hidden-file").unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + ".a-hidden-file").unwrap().is_file());

        fixture.teardown(true);
    }

    #[test]
    #[ignore]//docker
    fn get_file_by_key() {
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

        let path_path = Path::new(path).to_path_buf();
        let mut path_cache = command_assistors::PathCache::new(&path_path);

        // Changing directories.
        path_cache.switch();
        let output = Command::new("/ls-key/target/debug/lsk")
            .arg("get-file")
            .arg("/tmp/lsk_tests/")
            .arg("1")
            .output()
            .expect("failed to execute lsk process");


        path_cache.switch_back();

        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "/tmp/lsk_tests/a-dir\n"
        );

        fixture.teardown(true);
    }

    #[test]
    #[ignore]//docker
    fn get_file_by_key_in_dir() {
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

        let path_path = Path::new(path).to_path_buf();
        let mut path_cache = command_assistors::PathCache::new(&path_path);

        // Changing directories.
        path_cache.switch();

        let output_1 = Command::new("/ls-key/target/debug/lsk")
            .arg("get-file")
            .arg("1")
            .output()
            .expect("failed to execute lsk process");

        let output_2 = Command::new("/ls-key/target/debug/lsk")
            .arg("get-file")
            .arg("2")
            .output()
            .expect("failed to execute lsk process");

        let output_3 = Command::new("/ls-key/target/debug/lsk")
            .arg("get-file")
            .arg("3")
            .output()
            .expect("failed to execute lsk process");

        path_cache.switch_back();

        assert_eq!(
            String::from_utf8_lossy(&output_1.stdout),
            "/tmp/lsk_tests/a-dir\n"
        );

        assert_eq!(
            String::from_utf8_lossy(&output_2.stdout),
            "/tmp/lsk_tests/a-file\n"
        );

        assert_eq!(
            String::from_utf8_lossy(&output_3.stdout),
            ""
        );

        fixture.teardown(true);
    }

    #[test]
    #[ignore]//docker
    fn print_list_skip_hidden() {
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

        let path_path = Path::new(path).to_path_buf();
        let mut path_cache = command_assistors::PathCache::new(&path_path);

        // Changing directories.
        path_cache.switch();

        path_cache.switch_back();

        assert_eq!(true, metadata(path.to_string() + "a-dir").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "a-dir").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "a-dir").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + ".a-hidden-dir").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "a-file" ).unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + "a-dir/a-file").unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + "a-dir/b-file").unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + ".a-hidden-dir/a-file").unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + ".a-hidden-dir/.a-hidden-file").unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + ".a-hidden-file").unwrap().is_file());

        fixture.teardown(true);
    }
}
