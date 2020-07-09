use ls_key;
use std::fs::metadata;
use std::path::Path;
use ls_key::fixtures::{Fixture, command_assistors};
use std::process::Stdio;

// Linux's top level files and directories. The files have not content.
fn build_files(path: &str) {
        Fixture::new()
             .add_dirpath(path.to_string() + "arch")
             .add_dirpath(path.to_string() + "block")
             .add_dirpath(path.to_string() + "certs")
             .add_file(path.to_string() + "COPYING")
             .add_file(path.to_string() + "CREDITS")
             .add_dirpath(path.to_string() + "crypto")
             .add_dirpath(path.to_string() + "Documentation")
             .add_dirpath(path.to_string() + "drivers")
             .add_dirpath(path.to_string() + "fs")
             .add_dirpath(path.to_string() + "include")
             .add_dirpath(path.to_string() + "init")
             .add_dirpath(path.to_string() + "ipc")
             .add_file(path.to_string() + "Kbuild")
             .add_file(path.to_string() + "Kconfig")
             .add_dirpath(path.to_string() + "kernel")
             .add_dirpath(path.to_string() + "lib")
             .add_dirpath(path.to_string() + "LICENSES")
             .add_file(path.to_string() + "MAINTAINERS")
             .add_file(path.to_string() + "Makefile")
             .add_dirpath(path.to_string() + "mm")
             .add_dirpath(path.to_string() + "net")
             .add_file(path.to_string() + "README")
             .add_dirpath(path.to_string() + "samples")
             .add_dirpath(path.to_string() + "scripts")
             .add_dirpath(path.to_string() + "security")
             .add_dirpath(path.to_string() + "sound")
             .add_dirpath(path.to_string() + "tools")
             .add_dirpath(path.to_string() + "usr")
             .add_dirpath(path.to_string() + "virt")
             .build();

        // build usr/ subdir.
        let path_usr = path.to_string() + "usr/";
        Fixture::new()
             .add_file(path_usr.to_string() + "default_cpio_list")
             .add_file(path_usr.to_string() + "gen_init_cpio.c")
             .add_file(path_usr.to_string() + "gen_initramfs.sh")
             .add_dirpath(path_usr.to_string() + "include")
             .add_file(path_usr.to_string() + "initramfs_data.S")
             .add_file(path_usr.to_string() + "Kconfig")
             .add_file(path_usr.to_string() + "Makefile")
             .build();
}

fn assert_files(path: &str) {
        assert_eq!(true, metadata(path.to_string() + "arch").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "block").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "certs").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "COPYING").unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + "CREDITS").unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + "crypto").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "Documentation").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "drivers").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "fs").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "include").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "init").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "ipc").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "Kbuild").unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + "Kconfig").unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + "kernel").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "lib").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "LICENSES").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "MAINTAINERS").unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + "Makefile").unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + "mm").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "net").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "README").unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + "samples").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "scripts").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "security").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "sound").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "tools").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "usr").unwrap().is_dir());
        assert_eq!(true, metadata(path.to_string() + "virt").unwrap().is_dir());

        assert_eq!(true, metadata(path.to_string() + "usr/default_cpio_list").unwrap().is_file());
        assert_eq!(true, metadata(path.to_string() + "usr/include").unwrap().is_dir());
}

#[test]
#[ignore]//docker
fn list_build_files() {
        let path = "/tmp/lsk_tests/";
        build_files(path);
        let path_path = Path::new(path).to_path_buf();
        let mut path_cache = command_assistors::PathCache::new(&path_path);

        // Changing directories.
        path_cache.switch();

        path_cache.switch_back();

        assert_files(path);

        assert_eq!(true, Path::new(path).exists());

        Fixture::new()
            .add_dirpath(path.to_string())
            .teardown(true);

        assert_eq!(false, Path::new(path).exists())
}

#[test]
#[ignore]//docker
fn list() {
        let path = "/tmp/lsk_tests/";
        let list_all = false;
        build_files(path);
        let path_path = Path::new(path).to_path_buf();
        let mut path_cache = command_assistors::PathCache::new(&path_path);

        // Changing directories.
        path_cache.switch();
        let _ls_key = ls_key::LsKey::new(path, list_all, false, None, None, None);

        path_cache.switch_back();

        assert_files(path);

        assert_eq!(true, Path::new(path).exists());

        Fixture::new()
            .add_dirpath(path.to_string())
            .teardown(true);

        assert_eq!(false, Path::new(path).exists())
}

#[test]
#[ignore]//docker
fn fuzzy_list() {
        let input = "cr";
        let path = "/tmp/lsk_tests/";
        let list_all = false;
        build_files(path);
        let path_path = Path::new(path).to_path_buf();
        let mut path_cache = command_assistors::PathCache::new(&path_path);

        // Changing directories.
        path_cache.switch();

        let ls_key = ls_key::LsKey::new(path, list_all, false, None, None, None);

        let list_original = ls_key.list.clone();
        let ls_key_fuzzed = ls_key.clone().fuzzy_update(input.to_string());
        let list_fuzzed = ls_key_fuzzed.fuzzy_list.clone().unwrap();

        path_cache.switch_back();

        assert_files(path);

        assert_eq!(true, Path::new(path).exists());

        Fixture::new()
            .add_dirpath(path.to_string())
            .teardown(true);

        assert_eq!(false, Path::new(path).exists());

        assert_eq!(
            format!("{:#?}", list_fuzzed),
            //List {
            //     files: ["CREDITS"],
            //     dirs: ["security", "scripts", "crypto", "certs"],
            //     parent_dir: Some("/tmp/lsk_tests/"),
            //     path_history: ["/tmp/lsk_tests/"]
            //}
            format!("{}",
                    "List {\n    files: [\n        Entry {\n            path: \"security\",\n            file_type: Dir,\n            key: None,\n        },\n        Entry {\n            path: \"scripts\",\n            file_type: Dir,\n            key: None,\n        },\n        Entry {\n            path: \"crypto\",\n            file_type: Dir,\n            key: None,\n        },\n        Entry {\n            path: \"certs\",\n            file_type: Dir,\n            key: None,\n        },\n        Entry {\n            path: \"CREDITS\",\n            file_type: File,\n            key: None,\n        },\n    ],\n    parent_path: \"/tmp/lsk_tests/\",\n    path_history: [\n        \"/tmp/lsk_tests/\",\n    ],\n    filter: None,\n}"
            )
        );
        assert_ne!(
            format!("{:#?}", list_original),
            //list_fuzzed
            format!("{}",
                "List \
                 {\n    \
                     files: [\n        \"CREDITS\",\n    ],\n    \
                     dirs: [\n        \"security\",\n        \"scripts\",\n        \"crypto\",\n        \"certs\",\n    ],\n    \
                     parent_path: \"/tmp/lsk_tests/\",\n    \
                     path_history: [\n        \"/tmp/lsk_tests/\",\n    ],\n\
                 }"
            )
        );

}

#[test]
#[ignore]//docker
fn list_go_up_one_level() {
        Fixture::new()
            .add_dirpath("/tmp/lsk_tests/".to_string())
            .build();
        let path = "/tmp/lsk_tests/list_enter_dir/";
        let list_all = false;
        build_files(path);
        let path_path = Path::new(path).to_path_buf();
        let mut path_cache = command_assistors::PathCache::new(&path_path);

        // Changing directories.
        path_cache.switch();

        let mut ls_key = ls_key::LsKey::new(path, list_all, false, None, None, None);

        let list_original = ls_key.list.clone();

        //let key = "0";
        //let input =
        //    ls_key::Input {
        //         cmd: Some(key.to_string()),
        //         args: Some(vec![key.to_string()]),
        //         as_read: key.to_string(),
        //         cmd_type: Some(ls_key::CmdType::single_key)
        //    };
        //let input = ls_key::Input::new().parse(key.to_string());
        ls_key.list.parent_path.pop();
        let file_pathbuf = ls_key.list.parent_path.clone();
        ls_key.list.parent_path.pop();
        let list = ls_key.list.clone().update(file_pathbuf);
        ls_key.update(list);
        let list_up_level = ls_key.list;
        //self.run_list_read();

        path_cache.switch_back();

        assert_files(path);

        assert_eq!(true, Path::new(path).exists());

        Fixture::new()
            .add_dirpath("/tmp/lsk_tests/".to_string())
            .teardown(true);

        assert_eq!(false, Path::new(path).exists());

        assert_eq!(
            format!("{:#?}", list_up_level),
            "List {\n    files: [\n        Entry {\n            path: \"list_enter_dir\",\n            file_type: Dir,\n            key: None,\n        },\n    ],\n    parent_path: \"/tmp/lsk_tests\",\n    path_history: [\n        \"/tmp/lsk_tests/list_enter_dir/\",\n        \"/tmp/lsk_tests\",\n    ],\n    filter: None,\n}"
        );

        assert_ne!(list_original, list_up_level);
}

#[test]
#[ignore]//docker
fn list_enter_into_dir() {
        Fixture::new()
            .add_dirpath("/tmp/lsk_tests/".to_string())
            .build();
        let path = "/tmp/lsk_tests/list_enter_dir/";
        let list_all = false;
        build_files(path);
        let path_path = Path::new(path).to_path_buf();
        let mut path_cache = command_assistors::PathCache::new(&path_path);

        // Changing directories.
        path_cache.switch();

        let mut ls_key = ls_key::LsKey::new(path, list_all, false, None, None, None);

        let list_original = ls_key.list.clone();

        let is_fuzzed = false;
        let key: usize = 28;
        //let input =
        //    ls_key::Input {
        //         cmd: Some(key.to_string()),
        //         args: Some(vec![key.to_string()]),
        //         as_read: key.to_string(),
        //         cmd_type: Some(ls_key::CmdType::single_key)
        //    };

        let file_pathbuf = list_original.get_file_by_key(key, !is_fuzzed).unwrap();
        if metadata(file_pathbuf.clone()).unwrap().is_dir() {
            let list = ls_key.list.clone().update(file_pathbuf);
            ls_key.update(list);
        } else {
            assert!(false);
        }

        let list_enter_usr_dir = ls_key.list;

        path_cache.switch_back();

        assert_files(path);

        assert_eq!(true, Path::new(path).exists());

        Fixture::new()
            .add_dirpath("/tmp/lsk_tests/".to_string())
            .teardown(true);

        assert_eq!(false, Path::new(path).exists());

        assert_eq!(
            format!("{:#?}", list_enter_usr_dir),
            "List {\n    files: [\n        Entry {\n            path: \"Kconfig\",\n            file_type: File,\n            key: None,\n        },\n        Entry {\n            path: \"gen_init_cpio.c\",\n            file_type: File,\n            key: None,\n        },\n        Entry {\n            path: \"include\",\n            file_type: Dir,\n            key: None,\n        },\n        Entry {\n            path: \"Makefile\",\n            file_type: File,\n            key: None,\n        },\n        Entry {\n            path: \"initramfs_data.S\",\n            file_type: File,\n            key: None,\n        },\n        Entry {\n            path: \"gen_initramfs.sh\",\n            file_type: File,\n            key: None,\n        },\n        Entry {\n            path: \"default_cpio_list\",\n            file_type: File,\n            key: None,\n        },\n    ],\n    parent_path: \"/tmp/lsk_tests/list_enter_dir/usr\",\n    path_history: [\n        \"/tmp/lsk_tests/list_enter_dir/\",\n        \"/tmp/lsk_tests/list_enter_dir/usr\",\n    ],\n    filter: None,\n}"
        );

        assert_ne!(list_original, list_enter_usr_dir);
}

#[test]
#[ignore]//docker
fn list_enter_into_fuzzed_dir() {
        Fixture::new()
            .add_dirpath("/tmp/lsk_tests/".to_string())
            .build();
        let path = "/tmp/lsk_tests/list_enter_dir/";
        let list_all = false;
        build_files(path);
        let path_path = Path::new(path).to_path_buf();
        let mut path_cache = command_assistors::PathCache::new(&path_path);

        // Changing directories.
        path_cache.switch();

        let mut ls_key = ls_key::LsKey::new(path, list_all, false, None, None, None);

        let list_original = ls_key.list.clone();
        ls_key.list.parent_path.pop();
        let file_pathbuf = ls_key.list.parent_path.clone();
        ls_key.list.parent_path.pop();
        let list = ls_key.list.clone().update(file_pathbuf);
        ls_key.update(list);
        let list_up_level = ls_key.list;

        path_cache.switch_back();

        assert_files(path);

        assert_eq!(true, Path::new(path).exists());

        Fixture::new()
            .add_dirpath("/tmp/lsk_tests/".to_string())
            .teardown(true);

        assert_eq!(false, Path::new(path).exists());

        assert_eq!(
            format!("{:#?}", list_up_level),
            "List {\n    files: [\n        Entry {\n            path: \"list_enter_dir\",\n            file_type: Dir,\n            key: None,\n        },\n    ],\n    parent_path: \"/tmp/lsk_tests\",\n    path_history: [\n        \"/tmp/lsk_tests/list_enter_dir/\",\n        \"/tmp/lsk_tests\",\n    ],\n    filter: None,\n}"
        );

        assert_ne!(list_original, list_up_level);
}


#[test]
#[ignore]//play
fn list_test_cmd_in_different_dir() {
        let path = "/tmpt/lsk_tests/";
        build_files(path);
        let path_path = Path::new(path).to_path_buf();
        let mut path_cache = command_assistors::PathCache::new(&path_path);

        // Changing directories.
        path_cache.switch();

        //let cmd = "vim";
        //let args = [""];
        //std::process::Command::new(cmd)
        //    .args(&args)
        //    .spawn()
        //    .expect("failed to execute shell process.")
        //    .wait()
        //    .expect("unrecoverable failure to execute shell process.");

        let cmd = "fzf";
        let child = std::process::Command::new(cmd)
            //.args(&args)
            .stdout(Stdio::piped())
            .spawn()
            //.expect("failed to execute shell process.")
            //.wait()
            .expect("unrecoverable failure to execute shell process.");

        let output = child
            .wait_with_output()
            .expect("failed to wait on child");

        path_cache.switch_back();

        assert_files(path);

        assert_eq!(true, Path::new(path).exists());

        Fixture::new()
            .add_dirpath(path.to_string())
            .teardown(true);

        assert_eq!(false, Path::new(path).exists());

        let output: &str  = std::str::from_utf8(&output.stdout).unwrap();

        assert_eq!(
            output,
            "usr/default_cpio_list\n"
        );
}
//let list =
//     List {
//         files: ["COPYING", "Kconfig", "Makefile", "MAINTAINERS", "Kbuild", "CREDITS", "README"],
//         dirs: ["lsk_tests", "sound", "certs", "kernel", "include", "samples", "Documentation", "drivers", "fs", "block", "net", "arch", "crypto", "mm", "scripts", "tools", "init", "LICENSES", "virt", "ipc", "security", "lib", "usr"],
//         parent_path: "/tmp/lsk_tests/",
//         parent_dir: Some("/tmp/lsk_tests/"),
//         path_history: ["/tmp/lsk_tests/"]
//    };

#[cfg(test)]
mod list {
    use std::fs::metadata;
    use std::path::Path;
    use super::{ls_key::list::List, ls_key::list::print_list_with_keys, Fixture};

    #[test]
    #[ignore]//docker
    fn current_print_list_include_hidden() {
        let path = "./";

        let list = List::new(path)
            .list_include_hidden()
            .unwrap();

        print_list_with_keys(list).unwrap();
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

        let list = List::new(path)
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

        let list = List::new(path)
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
