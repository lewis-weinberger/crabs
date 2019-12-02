use crabs::{check_resize, levels, prompt_for_filename, save_to_ron, user_input};
use std::io::{stdout, Write};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor, style};

fn main() -> Result<(), std::io::Error> {
    // Prompt for filename to save to
    let filename = prompt_for_filename();

    // Initialise level
    let mut map = levels::blank_map();

    {
        // Initialise terminal
        let mut stdout = stdout()
            .into_raw_mode()
            .expect("Unable to initialise terminal");
        let mut stdin = termion::async_stdin().keys();
        write!(stdout, "{}{}", clear::All, cursor::Hide)?;

        // Determine initial terminal size
        let mut term_size: (u16, u16) = (0, 0);
        check_resize(&mut term_size);

        // User position
        let mut user: [usize; 2] = [map.dimensions[0] / 2, map.dimensions[1] / 2];

        // Game loop
        let mut complete = false;
        'game: while !complete {
            // Allow user to adjust map (input is asynchronous)
            match stdin.next() {
                Some(Ok(key)) => {
                    user_input(key, &mut user, &mut complete, &mut map);
                }
                _ => (),
            };

            // Check if terminal has been resized
            if check_resize(&mut term_size) {
                // Clear before redraw
                write!(stdout, "{}", clear::All)?;
            }

            // Display current state to stdout
            for (y, x, ch) in map.clone() {
                if y == user[0] && x == user[1] {
                    // Position cursor for user
                    write!(
                        stdout,
                        "{}{}",
                        cursor::Goto(user[1] as u16 + 1, user[0] as u16 + 1),
                        '+'
                    )?;
                } else {
                    // Display map
                    write!(stdout, "{}{}", cursor::Goto(x as u16 + 1, y as u16 + 1), ch)?;
                }
            }
        }

        // Reset stdout
        write!(
            stdout,
            "{}{}{}{}",
            clear::All,
            style::Reset,
            cursor::Goto(1, 1),
            cursor::Show
        )?;
    }

    // Save user's map
    save_to_ron(&filename, &map)?;

    Ok(())
}
