use atags::raw;
use core::{slice, str};

pub use atags::raw::{Core, Mem};

/// An ATAG.
#[derive(Debug, Copy, Clone)]
pub enum Atag {
    Core(raw::Core),
    Mem(raw::Mem),
    Cmd(&'static str),
    Unknown(u32),
    None,
}

impl Atag {
    /// Returns `Some` if this is a `Core` ATAG. Otherwise returns `None`.
    pub fn core(self) -> Option<Core> {
        if let Atag::Core(core) = self {
            Some(core)
        } else {
            None
        }
    }

    /// Returns `Some` if this is a `Mem` ATAG. Otherwise returns `None`.
    pub fn mem(self) -> Option<Mem> {
        if let Atag::Mem(mem) = self {
            Some(mem)
        } else {
            None
        }
    }

    /// Returns `Some` with the command line string if this is a `Cmd` ATAG.
    /// Otherwise returns `None`.
    pub fn cmd(self) -> Option<&'static str> {
        if let Atag::Cmd(cmd) = self {
            Some(cmd)
        } else {
            None
        }
    }
}

impl From<raw::Core> for Atag {
    fn from(core: raw::Core) -> Atag {
        Atag::Core(core)
    }
}

impl From<raw::Mem> for Atag {
    fn from(mem: raw::Mem) -> Atag {
        Atag::Mem(mem)
    }
}

impl From<&'static raw::Cmd> for Atag {
    fn from(cmd: &raw::Cmd) -> Atag {
        let mut len: usize = 0;
        let mut byte = &cmd.cmd as *const u8;

        while unsafe { *byte } != 0 {
            byte = unsafe { byte.add(1) };
            len += 1;
        }
        Atag::Cmd(&str::from_utf8(unsafe {
            slice::from_raw_parts(&cmd.cmd as *const u8, len)
        }).expect("Failed to convert `Atag::Cmd` to `str`."))
    }
}

impl<'a> From<&'static raw::Atag> for Atag {
    fn from(atag: &'static raw::Atag) -> Atag {
        unsafe {
            match (atag.tag, &atag.kind) {
                (raw::Atag::CORE, &raw::Kind { core }) => core.into(),
                (raw::Atag::MEM, &raw::Kind { mem }) => mem.into(),
                (raw::Atag::CMDLINE, &raw::Kind { ref cmd }) => cmd.into(),
                (raw::Atag::NONE, _) => Atag::None,
                (id, _) => Atag::Unknown(id),
            }
        }
    }
}
