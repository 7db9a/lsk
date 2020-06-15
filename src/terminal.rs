pub mod parent_shell {
    use::std::thread;
    use xdotool;
    use xdotool::keyboard;
    use xdotool::option_vec;
    use xdotool::optionvec::OptionVec;
    use xdotool::command::options::KeyboardOption;

    pub fn send_key<T: AsRef<str>>(key: T, delay: u32) {
        keyboard::send_key(key.as_ref(), option_vec![
            KeyboardOption::Delay(delay)
        ]);
    }

    pub fn type_text<T: AsRef<str>>(text: T, delay: u32/*, options: Option<KeyboardOption>*/) -> std::process::Output {
        if text.as_ref() == "BackSpace" {
             keyboard::send_key("BackSpace".as_ref(), option_vec![
                 KeyboardOption::Delay(delay)
             ])
        } else {
            keyboard::type_text(text.as_ref(), option_vec![
                KeyboardOption::Delay(delay)
            ])
        }
    }

    pub fn type_text_spawn(text: Vec<String>, delay: u32/*, options: Option<KeyboardOption>*/)  -> thread::JoinHandle<()> {
        //let text = text.as_ref().to_string();
	    let type_text = thread::spawn(move || {
	       // Send loop
	       // Send the message
           let text_iter = text.iter();
           let type_n_sleep = |text: String, delay: u32| {
               let few_ms = std::time::Duration::from_millis(100);
               std::thread::sleep(few_ms);
               if text == format!(r#""BackSpace""#) {
                   send_key("BackSpace", delay);
               } else {
                   type_text(text, delay);
               }
           };

           text_iter.for_each(|x|
               type_n_sleep(x.to_string(), delay)
           );
	    });
        //super::parent_shell::type_text(r#""$(printf 'cd $HOME && fzf \n ')""#);

	   type_text
    }
}
pub mod input_n_display {
    use std::path::{Path, PathBuf};
    use std::convert::TryFrom;
    use termion::input::TermRead;
    use termion::event::Key;
    use termion::raw::{IntoRawMode, RawTerminal};
    use termion::terminal_size;
    use term_grid::{/*Grid,*/ GridOptions, Direction, /*Display,*/ Filling, Cell};
    use std::io::{Read, Write, stdout, stdin, Stdout, StdoutLock};
    use termion::async_stdin;
    use termion::screen::AlternateScreen;
    use std::thread;
    use std::time::Duration;

    pub fn read() -> Result<(Option<String>), std::io::Error> {
        let stdout = stdout();
        let mut stdout = stdout.lock();
        let stdin = stdin();
        let mut stdin = stdin.lock();

        stdout.write_all(b":").unwrap();
        stdout.flush().unwrap();

        stdin.read_line()
    }

    // (columns/width, lines/height)
    pub fn size() -> (u16, u16) {
        terminal_size().unwrap()
    }

    pub fn alternate_screen() {
        {
            let mut screen = AlternateScreen::from(stdout());
            write!(screen, "Writing to alternat(iv)e screen!").unwrap();
            screen.flush().unwrap();
        }
        println!("Writing to main screen again.");
    }

    pub fn read_char_async() {
        let stdout = stdout();
        let mut stdout = stdout.lock().into_raw_mode().unwrap();
        let mut stdin = async_stdin().bytes();

        write!(stdout,
               "{}{}",
               termion::clear::All,
               termion::cursor::Goto(1, 1))
                .unwrap();

        loop {
            write!(stdout, "{}", termion::clear::CurrentLine).unwrap();

            let b = stdin.next();
            write!(stdout, "\r{:?}    <- This demonstrates the async read input char. Between each update a 100 ms. is waited, simply to demonstrate the async fashion. \n\r", b).unwrap();
            if let Some(Ok(b'q')) = b {
                break;
            }

            stdout.flush().unwrap();

            thread::sleep(Duration::from_millis(50));
            stdout.write_all(b"# ").unwrap();
            stdout.flush().unwrap();
            thread::sleep(Duration::from_millis(50));
            stdout.write_all(b"\r #").unwrap();
            write!(stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
            stdout.flush().unwrap();
        }
    }

    pub fn read_process_chars() -> Option<String> {
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

        fn write(some_stuff: &[u8], stdout: &mut RawTerminal<StdoutLock>, input_string: String) {
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
               termion::cursor::Goto(1, 1),
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

                match first {
                    'f' => write(b"fuzzy-widdle mode detected...", &mut stdout, input_string.clone()),
                    'r' => write(b"return file mode detected...", &mut stdout, input_string.clone()),
                    '$' => write(b"command mode detected... ", &mut stdout, input_string.clone()),
                    _ => write(b"invalid mode detected...", &mut stdout, input_string.clone()),
                };
            }

            stdout.flush().unwrap();

            if input.iter().last() == Some(&'\n') {
                input.pop();
                let input_string: String = input.iter().collect();
                result = Some(input_string);
                break
            }
        }

        write!(stdout, "{}", termion::cursor::Show).unwrap();
        result
    }

    pub fn read_char() {
        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode().unwrap();

        write!(stdout,
               "{}{}q to exit. Type stuff, use alt, and so on.{}",
               termion::clear::All,
               termion::cursor::Goto(1, 1),
               termion::cursor::Hide)
                .unwrap();
        stdout.flush().unwrap();

        for c in stdin.keys() {
            write!(stdout,
                   "{}{}",
                   termion::cursor::Goto(1, 1),
                   termion::clear::CurrentLine)
                    .unwrap();

            match c.unwrap() {
                Key::Char('q') => break,
                Key::Char(c) => {
                    match c {
                        //' ' => {
                        //    println!("$")
                        //},
                        //'v' => println!("{}im", c),
                        _ => println!("{}", c),
                    }
                }
                Key::Alt(c) => println!("^{}", c),
                Key::Ctrl(c) => println!("*{}", c),
                Key::Esc => println!("ESC"),
                Key::Left => println!("←"),
                Key::Right => println!("→"),
                Key::Up => println!("↑"),
                Key::Down => println!("↓"),
                Key::Backspace => println!("×"),
                _ => {}
            }
            stdout.flush().unwrap();
        }

        write!(stdout, "{}", termion::cursor::Show).unwrap();
    }

    pub use term_grid::{Grid, Display};

   // pub fn display(grid: Grid, width: usize) {
   //     println!("{}", grid.fit_into_width(width))
   //     //let grid = grid.fit_into_width(w));
   //     //if let Some(g) = grid {
   //     //    true
   //     //} else {
   //     //    false
   //     //}
   // }

    pub fn grid(entries: Vec<String>) -> Option<(Grid, usize)> {
        let mut grid = Grid::new(GridOptions {
                filling:     Filling::Spaces(3),
                direction:   Direction::LeftToRight,
        });

        for s in &entries
        {
                grid.add(Cell::from(s.as_str()));
        }

        //let stdout = stdout();
        //let mut stdout = stdout.lock();
        //let stdin = stdin();
        //let mut stdin = stdin.lock();

        let res = terminal_size();
        match res {
          Ok(r) => {
              let w = usize::from(r.0);
              let h = usize::from(r.1);

              Some((grid, w))
          },
          Err(_) => {
              None
          }
        }
    }

    pub fn grid_display(entries: Vec<String>) /*Result<(Grid), Error>*/{
        let mut grid = Grid::new(GridOptions {
                filling:     Filling::Spaces(3),
                direction:   Direction::LeftToRight,
        });

        for s in &entries
        {
                grid.add(Cell::from(s.as_str()));
        }

        let stdout = stdout();
        let mut stdout = stdout.lock();
        let stdin = stdin();
        let mut stdin = stdin.lock();

        let (w, h) = terminal_size()/*; match this     */.unwrap();
        /*match (w, h) {
            Ok((w, h)) => {
                let w = usize::from(w);
                let h = usize::from(h);
                grid.fit_into_width(w)
            },
            Err(_) => {
                None
            }
          }
        */

        let w = usize::from(w);
        let h = usize::from(h);

        println!("{}", grid.fit_into_width(w).unwrap());
    }
}

pub mod shell {
    use cmd_lib::{run_fun, info};

    pub fn spawn(cmd: String, args: Vec<String>) {
        std::process::Command::new(cmd)
            .args(args)
            .spawn()
            .expect("failed to execute shell process.")
            .wait()
            .expect("unrecoverable failure to execute shell process.");
    }

    pub fn cmd(cmd: String) -> Result<(String), std::io::Error> {
        run_fun!("{}", cmd)
    }
}

pub mod grid_display {
    pub fn grid() {
       use term_grid::{Grid, GridOptions, Direction, Filling, Cell};

       let mut grid = Grid::new(GridOptions {
               filling:     Filling::Spaces(7),
               direction:   Direction::LeftToRight,
       });

       for s in &["one", "two", "three", "four", "five", "six", "seven",
                  "eight", "nine", "ten", "eleven", "twelve"]
       {
               grid.add(Cell::from(*s));
       }

       println!("{}", grid.fit_into_width(50).unwrap());
    }

    pub fn grid_abnormal() {
       use term_grid::{Grid, GridOptions, Direction, Filling, Cell};

       let mut grid = Grid::new(GridOptions {
               filling:     Filling::Spaces(4),
               direction:   Direction::LeftToRight,
       });

       for s in &["one", "two"]
       {
               grid.add(Cell::from(*s));
       }

       println!("{}", grid.fit_into_width(24).unwrap());
    }

    pub fn grid_no_borrow() {
       use term_grid::{Grid, GridOptions, Direction, Filling, Cell};

       let mut grid = Grid::new(GridOptions {
               filling:     Filling::Spaces(1),
               direction:   Direction::LeftToRight,
       });

       for s in ["one".to_string(), "two".to_string(), "three".to_string(), "four".to_string(), "five".to_string(), "six".to_string(), "seven".to_string(),
                  "eight".to_string(), "nine".to_string(), "ten".to_string(), "eleven".to_string(), "twelve".to_string()].iter()
       {
               grid.add(Cell::from(s.clone()));
       }

       println!("{}", grid.fit_into_width(24).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::thread;

    //#[test]
    //#[ignore] // Need a spawn in a spawn.
    //fn xdotool_type_text() {
    //    println!("");
    //    println!("");
    //    let text_vec = vec![
    //         r#""$(printf 'cd $HOME && fzf \n ')""#.to_string(),
    //         r#""$(printf '1\n ')""#.to_string(),
    //         r#""$(printf 'cd - \n ')""#.to_string(),
    //    ];
    //    let spawn = super::parent_shell::type_text_spawn(text_vec, 50);
    //    spawn.join();
    //}

    //#[test]
    //#[ignore]//play
    fn termion_read_process_chars() {
	    let test_spawn = thread::spawn(move || {
            let result = super::input_n_display::read_process_chars();
            assert_eq!(result, Some("lift off!".to_string()));
	    });

        //let spawn = super::parent_shell::type_text_spawn(vec![r#""$(printf 'q \n ')""#.to_string()], 200);

        test_spawn.join();
        //spawn.join();
    }

    //#[test]
    //#[ignore]//play
    fn termion_alternate_screen() {
	    let test_spawn = thread::spawn(move || {
            super::input_n_display::alternate_screen()
	    });

        //let spawn = super::parent_shell::type_text_spawn(vec![r#""$(printf 'q \n ')""#.to_string()], 200);

        test_spawn.join();
        //spawn.join();
    }

    #[test]
    #[ignore]//play
    fn termion_key() {
	    let test_spawn = thread::spawn(move || {
            super::input_n_display::read_char()
	    });

        //let spawn = super::parent_shell::type_text_spawn(vec![r#""$(printf 'q \n ')""#.to_string()], 200);

        test_spawn.join();
        //spawn.join();
    }

    //#[test]
    #[ignore]//play
    fn termion_async_key() {
	    let test_spawn = thread::spawn(move || {
            super::input_n_display::read_char_async()
	    });

        //let spawn = super::parent_shell::type_text_spawn(vec![r#""$(printf 'q \n ')""#.to_string()], 200);

        test_spawn.join();
        //spawn.join();
    }

    #[test]
    #[ignore]//docker
    fn tterminal_size_with_termion() {
        let (w, h) = super::input_n_display::size();
        println!("\nwidth: {}\nheight: {}", w, h);
    }
    #[test]
    #[ignore]//play
    fn takes_input_read() {
        println!("");
        let spawn = super::parent_shell::type_text_spawn(vec![r#""$(printf 'hello \n ')""#.to_string()], 200);
        spawn.join();
        super::input_n_display::read();
    }

    #[test]
    #[ignore]//docker
    fn display_grid() {
        println!("");
        println!("");
        super::grid_display::grid();
        super::grid_display::grid_no_borrow()
    }

    #[test]
    #[ignore]//docker
    fn terminal_grid() {
        let entry = "entry".to_string();

        let mut entries: Vec<String> = vec![];

        for _ in 0..49 {
            entries.push(entry.clone())
        }
        println!("");
        super::input_n_display::grid_display(entries);
    }
}
