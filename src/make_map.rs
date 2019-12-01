use crabs::{user_input, check_resize, levels, prompt_for_filename, save_to_ron};
use std::io::{Write, stdout};
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::{clear, style, cursor};

fn main() {
    // Initialise level
    let mut map = levels::blank_map();

    {
        // Initialise terminal
        let mut stdout = stdout().into_raw_mode()
            .expect("Unable to initialise terminal");
        let mut stdin = termion::async_stdin().keys();
        match write!(stdout, "{}{}", clear::All, cursor::Hide) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Error writing to stdout: {:?}", err.kind());
            }
        }

        // Determine initial terminal size
        let mut term_size: (u16, u16) = (0, 0);
        check_resize(&mut term_size);

        // User position
        let mut user: [usize; 2] = [12, 40];

        // Game loop
        let mut complete = false;
        'game: while !complete {
            // Allow user to adjust map (input is asynchronous)
            match stdin.next() {
                Some(Ok(key)) => {
                    user_input(key, &mut user, &mut complete, &mut map);
                },
                _ => (),
            };

            // Check if terminal has been resized
            if check_resize(&mut term_size) {
                // Clear before redraw
                match write!(stdout, "{}", clear::All) {
                    Ok(_) => (),
                    Err(err) => {
                        eprintln!("Error writing to stdout: {:?}", err.kind());
                    }
                }
            }

            // Display current state to stdout
            for (y, x, ch) in map.clone() {
                if y == user[0] && x == user[1] {
                    // Position cursor for user
                    match write!(stdout, "{}{}", cursor::Goto(user[1] as u16 + 1, user[0] as u16 + 1), '+') {
                        Ok(_) => (),
                        Err(err) => {
                            eprintln!("Error writing to stdout: {:?}", err.kind());
                            break 'game;
                        }
                    }
                } else {
                    // Display map
                    match write!(stdout, "{}{}", cursor::Goto(x as u16 + 1, y as u16 + 1), ch) {
                        Ok(_) => (),
                        Err(err) => {
                            eprintln!("Error writing to stdout: {:?}", err.kind());
                            break 'game;
                        }
                    }
                }
            }
        }

        // Reset stdout
        match write!(stdout, "{}{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1), cursor::Show) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Error writing to stdout: {:?}", err.kind());
            }
        }
    }
    
    // Prompt for filename to save to
    let filename = prompt_for_filename();

    // Save user's map
    save_to_ron(&filename, &map);
}
