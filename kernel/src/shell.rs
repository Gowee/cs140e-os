use stack_vec::StackVec;
use console::{kprint, kprintln, CONSOLE};
use std;
use pi;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs,
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>,
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns: it is perpetually in a shell loop.
pub fn shell(prefix: &str) -> ! {
    use std::io::Write;
    kprintln!("Welcome to...");
    kprintln!(
        "\
███████╗ ██████╗ ███████╗
██╔════╝██╔═══██╗██╔════╝
███████╗██║   ██║███████╗
╚════██║██║   ██║╚════██║
███████║╚██████╔╝███████║
╚══════╝ ╚═════╝ ╚══════╝"
    );
    loop {
        let mut byte;
        let mut input = [0u8; 512];
        let mut index = 0;
        { // occupying console
            let mut console = CONSOLE.lock();
            console.write(prefix.as_bytes()).unwrap();
            while {
                byte = console.read_byte();
                !(byte == b'\n' || byte == b'\r') // end of line
            }
            {
                if index == 512 {
                    // command can only be at most 512 bytes in length
                    console.write_byte(b'\x07'); // bell
                    continue;
                }
                match byte {
                    b'\x08' | b'\x7F' => {
                        // BS or DEL
                        if index > 0 {
                            index -= 1;
                            console.write(b"\x08 \x08").unwrap(); // erase a character
                        } else {
                            console.write_byte(b'\x07'); // illegal backspacing, bell
                        }
                    }
                    byte @ b'\x20'...b'\x7E' => {
                        // printable charaters
                        console.write_byte(byte);
                        input[index] = byte;
                        index += 1;
                    }
                    _ => {
                        // unrecognizable characters
                        console.write_byte(b'\x07'); // bell
                    }
                }
            }
        }
        kprintln!();

        let mut storage = [""; 64];
        match Command::parse(std::str::from_utf8(&input[..index]).unwrap(), &mut storage) {
            Err(Error::Empty) => {
                continue;
            }
            Err(Error::TooManyArgs) => kprintln!("error: too many arguments"),
            Ok(command) => {
                match command.path() {
                    "echo" => {
                        for arg in command.args[1..].iter() {
                            kprint!("{} ", arg);
                        }
                        kprintln!();
                    },
                    "atags" => {
                        for atag in pi::atags::Atags::get() {
                            kprintln!("{:#?}", atag)
                        }
                    },
                    _ => {
                        kprintln!("unknown command: {command}", command = command.path());
                    }
                }
            }
        }

    }
}
