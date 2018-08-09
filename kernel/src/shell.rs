use console::{kprint, kprintln, CONSOLE};
use fat32::traits::{Dir, Entry, File, FileSystem};
use fat32::vfat;
use pi;
use stack_vec::StackVec;
use std;
use std::io::Read;
use std::path::PathBuf;

use super::FILE_SYSTEM;

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
    let mut cwd = PathBuf::from("/");
    loop {
        let mut byte;
        let mut input = [0u8; 512];
        let mut index = 0;
        {
            // occupying console
            let mut console = CONSOLE.lock();
            console.write_byte(b'(');
            console
                .write_fmt(format_args!("{}", cwd.display()))
                .unwrap();
            console.write_byte(b')');
            console.write(prefix.as_bytes()).unwrap();
            while {
                byte = console.read_byte();
                !(byte == b'\n' || byte == b'\r') // end of line
            } {
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
            Err(Error::TooManyArgs) => kprintln!("Error: Too many arguments."),
            Ok(command) => match command.path() {
                "echo" => {
                    for arg in command.args[1..].iter() {
                        kprint!("{} ", arg);
                    }
                    kprintln!();
                }
                "atags" => {
                    for atag in pi::atags::Atags::get() {
                        kprintln!("{:#?}", atag);
                    }
                }
                "pwd" => {
                    let mut console = CONSOLE.lock();
                    console
                        .write_fmt(format_args!("{}", cwd.display()))
                        .unwrap();
                    console.write(b"\r\n").unwrap();
                }
                "cd" => {
                    if command.args.len() == 2 {
                        match command.args[1] {
                            "." => {}
                            ".." => {
                                cwd.pop();
                            }
                            directory => match FILE_SYSTEM.open(cwd.join(directory)) {
                                Ok(vfat::Entry::File(_file)) => {
                                    kprintln!("Error: Cannot cd into a file.");
                                }
                                Ok(vfat::Entry::Dir(_dir)) => {
                                    kprintln!("");
                                    cwd.push(command.args[1]);
                                }
                                Err(err) => {
                                    kprintln!("{:?}", err);
                                }
                            },
                        }
                    } else {
                        kprintln!("Error: Expected exactly 1 argument.");
                    }
                }
                "ls" => {
                    let (all, path) = match command.args.len() {
                        1 => (false, cwd.clone()),
                        2 => {
                            if command.args[1] == "-a" {
                                match FILE_SYSTEM.open(cwd.join("-a")) {
                                    Ok(_) => (false, cwd.join("-a")),
                                    Err(_) => (true, cwd.clone()),
                                }
                            } else {
                                (false, cwd.join(command.args[1]))
                            }
                        }
                        3 => {
                            if command.args[1] != "-a" {
                                kprintln!("Error: Invalid arguments.\nUsage: ls [-a] [DIRECOTRY]");
                                continue;
                            }
                            (true, cwd.join(command.args[2]))
                        }
                        _ => {
                            kprintln!(
                                "Error: Expected at most 2 arguments.\nUsage: ls [-a] [DIRECOTRY]"
                            );
                            continue;
                        }
                    };
                    match FILE_SYSTEM.open(&path) {
                        Ok(vfat::Entry::File(_file)) => {
                            kprintln!("Error: {} is a file.", path.to_str().unwrap_or("This"));
                        }
                        Ok(vfat::Entry::Dir(dir)) => {
                            for entry in dir.entries().expect("Read entries.") {
                                let metadata = entry.metadata();
                                let attr = &metadata.attributes;
                                if attr.hidden() && !all {
                                    continue;
                                }
                                let mut console = CONSOLE.lock();
                                console.write_byte(if attr.read_only() { b'r' } else { b'w' });
                                console.write_byte(if attr.hidden() { b'h' } else { b'v' });
                                console.write_byte(if attr.system() { b's' } else { b'-' });
                                console.write_byte(if attr.volume_id() { b'i' } else { b'-' });
                                console.write_byte(if entry.is_file() { b'f' } else { b'd' });
                                console.write_byte(if attr.archive() { b'a' } else { b'-' });
                                console.write_byte(b'\t');
                                console
                                    .write_fmt(format_args!(
                                        "{}\t{}\t{}\t{}{}\r\n",
                                        metadata.created_time,
                                        metadata.modified_time,
                                        match entry {
                                            vfat::Entry::File(ref file) => file.size(),
                                            vfat::Entry::Dir(ref _dir) => 0,
                                        },
                                        entry.name(),
                                        if entry.is_file() { "" } else { "/" }
                                    )).unwrap();
                            }
                        }
                        Err(err) => {
                            kprintln!("{:?}", err);
                        }
                    }
                }
                "cat" => {
                    let mut files = vec![];
                    for arg in command.args[1..].iter() {
                        match FILE_SYSTEM.open(cwd.join(arg)) {
                            Ok(vfat::Entry::Dir(_dir)) => {
                                kprintln!("Error: {} is a directory.", arg);
                                break;
                            }
                            Ok(vfat::Entry::File(file)) => {
                                files.push(file);
                            }
                            Err(err) => {
                                kprintln!("{:?}", err);
                                break;
                            }
                        }
                    }
                    for mut file in files {
                        let mut buf = vec![];
                        if let Err(err) = file.read_to_end(&mut buf) {
                            kprintln!("{:?}", err);
                            break;
                        }
                        kprint!("{}", &String::from_utf8_lossy(&buf[..]));
                        //CONSOLE.lock().write(&buf).unwrap(); // TODO: No \r is outputed? WTF???
                    }
                }
                _ => {
                    kprintln!(
                        "Error: Unknown command: {command}.",
                        command = command.path()
                    );
                }
            },
        }
    }
}
