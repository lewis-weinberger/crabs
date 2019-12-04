use std::io::{stdout, Write};
use std::{env, thread, time};

use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, color, cursor, style};

use crabs::{check_resize, process_args, user_input, Colour, TICK_TIME};

fn main() -> Result<(), std::io::Error> {
    // Process command line arguments
    let mut target_tick_time: time::Duration = TICK_TIME;
    let levels = process_args(env::args(), &mut target_tick_time);

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
        let mut user = [map.dimensions[0] / 2, map.dimensions[1] / 2];

        // Game loop
        let mut complete = false;
        let mut reset = false;
        'game: while !complete {
            let start_time = time::Instant::now();

            // Check if level needs to be reset
            if reset {
                crabs = level.0.clone();
                map = level.1.clone();
                user = [map.dimensions[0] / 2, map.dimensions[1] / 2];
                reset = false;
            }

            // Ensure that map is crab-free
            map.decrab();

            // Crabs are advanced every RATE number of game loops
            crabs.evolve(&mut map, &mut complete);

            // Allow user to adjust map (input is asynchronous)
            stdin.next().and_then(|res| {
                res.ok()
                    .map(|key| user_input(key, &mut user, &mut complete, &mut reset, &mut map))
            });

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

            let tick_time = time::Instant::now() - start_time;
            if tick_time < target_tick_time {
                thread::sleep(target_tick_time - tick_time);
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
