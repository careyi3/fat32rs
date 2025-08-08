use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use tempfile::NamedTempFile;

use fat32rs::disk::{BlockIO, Disk, IOError, Result};

pub struct FileBackedDevice {
    file: NamedTempFile,
}

impl FileBackedDevice {
    pub fn new(file: NamedTempFile) -> Self {
        Self { file }
    }
}

impl BlockIO for FileBackedDevice {
    fn read_block(&mut self, byte_offset: u64) -> Result<[u8; 512]> {
        self.file
            .seek(SeekFrom::Start(byte_offset))
            .map_err(|_| IOError::ReadError)?;

        let mut buf = [0u8; 512];
        self.file
            .read_exact(&mut buf)
            .map_err(|_| IOError::ReadError)?;
        Ok(buf)
    }

    fn write_block(&mut self, byte_offset: u64, data: [u8; 512]) -> Result<()> {
        self.file
            .seek(SeekFrom::Start(byte_offset))
            .map_err(|_| IOError::WriteError)?;
        self.file
            .write_all(&data)
            .map_err(|_| IOError::WriteError)?;
        self.file.flush().map_err(|_| IOError::WriteError)?;
        Ok(())
    }
}

fn setup_temp_disk() -> NamedTempFile {
    let fixture_path = Path::new("tests/data/drive.img");

    let mut fixture_file =
        File::open(fixture_path).expect("Fixture file missing: tests/data/drive.img");

    let mut temp_file = NamedTempFile::new().expect("Could not create temp file");

    // Copy contents
    std::io::copy(&mut fixture_file, temp_file.as_file_mut())
        .expect("Failed to copy fixture to temp file");

    // Optionally seek back to the start (if needed)
    temp_file.as_file_mut().seek(SeekFrom::Start(0)).unwrap();

    return temp_file;
}

pub fn disk() -> Disk<FileBackedDevice> {
    let tempfile = setup_temp_disk();

    let device = FileBackedDevice::new(tempfile);
    let disk = Disk::new(device);
    return disk;
}
