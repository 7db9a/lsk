pub mod parent_shell {
    use::std::thread;
    use xdotool;
    use xdotool::keyboard;
    use xdotool::option_vec;
    use xdotool::optionvec::OptionVec;
    use xdotool::command::options::KeyboardOption;

    pub fn send_key<T: AsRef<str>>(key: T) {
        keyboard::send_key("Return", option_vec![
            KeyboardOption::Delay(200)
        ]);
    }

    pub fn type_text<T: AsRef<str>>(text: T, delay: u32/*, options: Option<KeyboardOption>*/) -> std::process::Output {
        keyboard::type_text(text.as_ref(), option_vec![
            KeyboardOption::Delay(delay)
        ])
    }

    pub fn type_text_spawn(text: Vec<String>, delay: u32/*, options: Option<KeyboardOption>*/)  -> thread::JoinHandle<()> {
        //let text = text.as_ref().to_string();
	    let type_text = thread::spawn(move || {
	       // Send loop
	       // Send the message
           let text_iter = text.iter();
           let type_n_sleep = |text: String, delay: u32| {
               let few_ms = std::time::Duration::from_millis(1000);
               std::thread::sleep(few_ms);
               type_text(text, delay);
           };

           text_iter.for_each(|x|
               type_n_sleep(x.to_string(), delay)
           );
	    });
        //super::parent_shell::type_text(r#""$(printf 'cd $HOME && fzf \n ')""#);

	   type_text
    }
}
pub mod termion {
    use termion::input::TermRead;
    use termion::event::Key;
    use termion::raw::IntoRawMode;
    use termion::terminal_size;
    use std::io::{Write, stdout, stdin};

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
                        ' ' => {
                            println!("$")
                        },
                        'v' => println!("{}im", c),
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

    pub fn cmd(cmd: String) -> Result<(), std::io::Error> {
        let output  = run_fun!("{}", cmd).unwrap();
        info!("{}", output);

        Ok(())
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


// Uses termion for input and term_grid for display.
pub mod terminal_n_grid {
    use std::path::{Path, PathBuf};
    use termion::input::TermRead;
    use termion::terminal_size;
    use term_grid::{/*Grid,*/ GridOptions, Direction, /*Display,*/ Filling, Cell};
    use std::io::{Write, stdout, stdin};

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

    pub fn _grid(entries: Vec<String>) -> Option<(Grid, usize)> {
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

    pub fn grid(entries: Vec<String>) /*Result<(Grid), Error>*/{
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

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::thread;

    //#[test]
    #[ignore] // Need a spawn in a spawn.
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

    #[test]
    #[ignore]//host
    fn termion_key() {
	    let test_spawn = thread::spawn(move || {
            super::termion::read_char()
	    });

        let spawn = super::parent_shell::type_text_spawn(vec![r#""$(printf 'q \n ')""#.to_string()], 200);

        test_spawn.join();
        spawn.join();
    }

    #[test]
    #[ignore]//docker
    fn tterminal_size_with_termion() {
        let (w, h) = super::termion::size();
        println!("\nwidth: {}\nheight: {}", w, h);
    }
    #[test]
    #[ignore]//host
    fn takes_input_read() {
        println!("");
        let spawn = super::parent_shell::type_text_spawn(vec![r#""$(printf 'hello \n ')""#.to_string()], 200);
        spawn.join();
        super::termion::read();
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
        super::terminal_n_grid::grid(entries);
    }
}
