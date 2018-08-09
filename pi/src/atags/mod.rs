mod raw;
mod atag;

pub use self::atag::*;

/// The address at which the firmware loads the ATAGS.
const ATAG_BASE: usize = 0x100;

/// An iterator over the ATAGS on this system.
pub struct Atags {
    ptr: &'static raw::Atag,
}

impl Atags {
    /// Returns an instance of `Atags`, an iterator over ATAGS on this system.
    pub fn get() -> Atags {
        Atags { ptr: unsafe { &*(ATAG_BASE as *const raw::Atag) } }
    }
}

impl Iterator for Atags {
    type Item = Atag;

    fn next(&mut self) -> Option<Atag> {
        let ret = Some(self.ptr.into()); // `From<&raw::Atag> for atag::Atag` implemented
        self.ptr = self.ptr.next()?; // If the next does not exist, then `self.ptr.tag == raw::Atag::NONE` must hold and all informative atags have already been produced. In this case, iteration ends without producing a `raw::Atag::NONE`. This is different from raw::Atag::next.
        ret
    }
}
