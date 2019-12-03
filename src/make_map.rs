use crabs::{
    check_resize, levels, prompt_for_filename, prompt_for_positions, prompt_for_velocities,
    save_to_ron, user_input, Entities,
};
use std::io::{stdout, Write};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{clear, cursor, style};

fn main() -> Result<(), std::io::Error> {
    // Prompt for filenames
    let filename = prompt_for_filename()?;

    // Prompt for crabs
    let positions = prompt_for_positions()?;
    let velocities = prompt_for_velocities()?;

    let crabs = if positions.len() == velocities.len() {
        Entities::new(positions.clone(), velocities.clone())
    } else {
        Entities::new(Vec::new(), Vec::new())
    };

    // Initialise level
    let mut map = levels::blank_map();
    map.instantaneous(&crabs);

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
    let mut reset = false;
    'game: while !complete {
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

    // Save user's map
    map.decrab();
    if positions.len() == velocities.len() {
        save_to_ron(&filename, &map, positions, velocities)?;
    } else {
        save_to_ron(&filename, &map, Vec::new(), Vec::new())?;
    }

    Ok(())
}
