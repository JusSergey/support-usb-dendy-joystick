#![feature(vec_remove_item)]
use std::fs::File;
use std::io::Read;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::result::Result;
use std::process::Command;

type RawCode = [u8; 8];

enum Error {
    InvalidKey
}

#[derive(Copy, Clone)]
enum Button {
    Start,
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    C,
    X,
    Y,
    Z,
}

impl Display for Button {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let alpha = match self {
            Self::Start => "s",
            Self::Up => "u",
            Self::Down => "d",
            Self::Left => "q",
            Self::Right => "r",
            Self::A => "a",
            Self::B => "w",
            Self::C => "aw", // the same as 'A' button
            Self::X => "x",
            Self::Y => "y",
            Self::Z => "xy",
        };
        write!(f, "{}", alpha)
    }
}

enum Action {
    Push,
    Release,
}

impl ToString for Action {
    fn to_string(&self) -> String {
            match self {
                Self::Push => "keydown",
                Self::Release => "keyup",
            }.to_string()
    }
}

struct ButtonUniform {
    pushed_buttons: Vec<u8>,
    last_horizontal_arrow_button: Button,
    last_vertical_arrow_button: Button,
}

impl ButtonUniform {

    pub fn new() -> Self {
        Self {
            // the first values doesn't matter, but anyway
            // we have to initialize them
            last_vertical_arrow_button: Button::Down,
            last_horizontal_arrow_button: Button::Left,
            pushed_buttons: Vec::new(),
        }
    }

    pub fn convert_to_uniform(&mut self, code: isize) -> Result<(Button, Action), Error> {
        match code {
            16777481 => Ok((Button::Start, Action::Push)),
            265 => Ok((Button::Start, Action::Release)),

            16777474 => Ok((Button::A, Action::Push)),
            258 => Ok((Button::A, Action::Release)),

            16777473 => Ok((Button::B, Action::Push)),
            257 => Ok((Button::B, Action::Release)),

            16777477 => Ok((Button::C, Action::Push)),
            261 => Ok((Button::C, Action::Release)),

            16777475 => Ok((Button::X, Action::Push)),
            259 => Ok((Button::X, Action::Release)),

            16777472 => Ok((Button::Y, Action::Push)),
            256 => Ok((Button::Y, Action::Release)),

            16777476 => Ok((Button::Z, Action::Push)),
            260 => Ok((Button::Z, Action::Release)),

            // up-push
            25166337 => {
                self.last_vertical_arrow_button = Button::Up;
                Ok((self.last_vertical_arrow_button, Action::Push))
            },

            // down-push
            4286513665 => {
                self.last_vertical_arrow_button = Button::Down;
                Ok((self.last_vertical_arrow_button, Action::Push))
            }

            // left-push
            25166336 => {
                self.last_horizontal_arrow_button = Button::Left;
                Ok((self.last_horizontal_arrow_button, Action::Push))
            }

            // right-push
            4286513664 => {
                self.last_horizontal_arrow_button = Button::Right;
                Ok((self.last_horizontal_arrow_button, Action::Push))
            }

            // last up/down -release
            513 => Ok((self.last_vertical_arrow_button, Action::Release)),

            // last left/right -release
            512 => Ok((self.last_horizontal_arrow_button, Action::Release)),

            _ => Err(Error::InvalidKey),
        }
    }

    fn xdo(&mut self, code: isize) {
        if let Ok((button, action)) = self.convert_to_uniform(code) {

            for symbol in button.to_string().chars() {
                let symbol = symbol as u8;
                match action {
                    Action::Push => {

                        if self.pushed_buttons.contains(&symbol) {
                            self.pushed_buttons.push(symbol);
                            continue;
                        }
                        self.pushed_buttons.push(symbol);
                    },
                    Action::Release => {
                        self.pushed_buttons.remove_item(&symbol);
                        if self.pushed_buttons.contains(&symbol) {
                            continue;
                        }
                    },
                }

                println!("run {} {}", action.to_string(), (symbol as char).to_string());

                match Command::new("xdotool")
                        .arg(action.to_string())
                        .arg((symbol as char).to_string())
                        .spawn()
                {
                    Ok(mut handle) => if let Err(_) = handle.wait() {
                        println!("waiting to the end of process failed.");
                    },
                    Err(_) => println!("Failed to run xdotool"),
                }
            }

            // println!("run xdotool {} {}", action.to_string(), button.to_string());
            // match
            // Command::new("xdotool")
            //     .arg(action.to_string())
            //
            //     .arg(button.to_string())
            //
            //     .spawn()
            // {
            //     Ok(mut handle) => if let Err(_) = handle.wait() {
            //         println!("waiting to the end of process failed.");
            //     },
            //     Err(_) => println!("Failed to run xdotool"),
            // }
        } else {
            println!("Error. invalid key");
        }
    }

}



fn main() -> Result<(), std::io::Error> {
    let mut file = File::open("/dev/input/js0")?;
    let mut uniform = ButtonUniform::new();

    loop {
        let mut raw_code: RawCode = [0; 8];

        match file.read(&mut raw_code) {
            Ok(_) => {
                println!("raw => {:?}", raw_code);
                raw_code[0] = 0;
                raw_code[1] = 0;
                raw_code[2] = 0;
                raw_code[3] = 0;
                uniform.xdo(isize::from_be_bytes(raw_code));
            },
            Err(err) => {
                return Err(err);
            }
        }
    }
}
