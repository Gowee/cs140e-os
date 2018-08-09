pub mod sd;

use std::io;
use std::path::Path;

use console::kprintln;
pub use fat32::traits;
use fat32::vfat::{Dir, Entry, File, Shared, VFat};

use self::sd::Sd;
use mutex::Mutex;

pub use self::sd::wait_micros;

pub struct FileSystem(Mutex<Option<Shared<VFat>>>);

impl FileSystem {
    /// Returns an uninitialized `FileSystem`.
    ///
    /// The file system must be initialized by calling `initialize()` before the
    /// first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        FileSystem(Mutex::new(None))
    }

    /// Initializes the file system.
    ///
    /// # Panics
    ///
    /// Panics if the underlying disk or file sytem failed to initialize.
    pub fn initialize(&self) {
        *self.0.lock() = Some(
            VFat::from(Sd::new().expect("Initialize SD card driver."))
                .expect("Initialize VFat for SD Card."),
        );
    }
}

// Does not work due to borrow issues.
/*
impl Deref for FileSystem {
    type Target = Shared<VFat>;

    fn deref(&self) -> &Shared<VFat> {
        self.0.lock().as_ref().expect("Filesystem has not been initialized.")
    }
}

impl DerefMut for FileSystem {
    fn deref_mut(&mut self) -> &mut Shared<VFat> {
        self.0.lock().as_mut().expect("Filesystem has not been initialized.")
    }
}*/

impl<'a> traits::FileSystem for &'a FileSystem {
    type File = File;
    type Dir = Dir;
    type Entry = Entry;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        self.0
            .lock()
            .as_ref()
            .expect("Filesystem is initialized.")
            .open(path)
    }

    fn create_file<P: AsRef<Path>>(self, _path: P) -> io::Result<Self::File> {
        unimplemented!("read only file system")
    }

    fn create_dir<P>(self, _path: P, _parents: bool) -> io::Result<Self::Dir>
    where
        P: AsRef<Path>,
    {
        unimplemented!("read only file system")
    }

    fn rename<P, Q>(self, _from: P, _to: Q) -> io::Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        unimplemented!("read only file system")
    }

    fn remove<P: AsRef<Path>>(self, _path: P, _children: bool) -> io::Result<()> {
        unimplemented!("read only file system")
    }
}
