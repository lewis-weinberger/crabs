use crabs::{check_resize, process_args, user_input, Colour, RATE};
use std::env;
use std::io::{stdout, Write};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, color, cursor, style};

fn main() -> Result<(), std::io::Error> {
    // Process command line arguments
    let levels = process_args(env::args());

    // Initialise terminal
    let mut stdout = stdout().into_raw_mode()?;
    let mut stdin = termion::async_stdin().keys();
    write!(stdout, "{}{}", clear::All, cursor::Hide)?;

    // Determine initial terminal size
    let mut term_size: (u16, u16) = (0, 0);
    check_resize(&mut term_size);

    // Loop over levels
    for level in levels.iter() {
        // Initialise level
        let (mut crabs, mut map) = level.clone();

        // User position
        let mut user: [usize; 2] = [map.dimensions[0] / 2, map.dimensions[1] / 2];

        // Game loop
        let mut complete = false;
        let mut loop_count = 1;
        'game: while !complete {
            if loop_count == 0 {
                // Ensure that map is crab-free
                map.decrab();

                // Crabs are advanced every RATE number of game loops
                crabs.evolve(&mut map, &mut complete);
            }
            loop_count = (loop_count + 1) % RATE;

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
                        "{}{}{}{}",
                        cursor::Goto(user[1] as u16 + 1, user[0] as u16 + 1),
                        color::Fg(color::Green),
                        '+',
                        color::Fg(color::Reset)
                    )?;
                } else {
                    // Display map
                    write!(
                        stdout,
                        "{}{}{}{}",
                        cursor::Goto(x as u16 + 1, y as u16 + 1),
                        ch.to_fg_colour(),
                        ch,
                        color::Fg(color::Reset)
                    )?;
                }
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

    // Successful return
    Ok(())
}
