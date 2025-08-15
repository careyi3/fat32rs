use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use fat32rs::disk::{BlockIO, Disk, Error, Result};

pub struct FsFileBackedDevice {
    file: File,
}

impl FsFileBackedDevice {
    pub fn new(file: File) -> Self {
        Self { file }
    }
}

impl BlockIO for FsFileBackedDevice {
    fn read_block(&mut self, byte_offset: u64) -> Result<[u8; 512]> {
        self.file
            .seek(SeekFrom::Start(byte_offset))
            .map_err(|_| Error::ReadError)?;

        let mut buf = [0u8; 512];
        self.file
            .read_exact(&mut buf)
            .map_err(|_| Error::ReadError)?;
        Ok(buf)
    }

    fn write_block(&mut self, byte_offset: u64, data: [u8; 512]) -> Result<()> {
        self.file
            .seek(SeekFrom::Start(byte_offset))
            .map_err(|_| Error::WriteError)?;
        self.file.write_all(&data).map_err(|_| Error::WriteError)?;
        self.file.flush().map_err(|_| Error::WriteError)?;
        Ok(())
    }
}

fn open_path_rw(path: &Path) -> std::io::Result<File> {
    OpenOptions::new().read(true).write(true).open(path)
}

fn disk_from_path(path: &Path) -> Disk<FsFileBackedDevice> {
    let file = open_path_rw(path).expect("Failed to open provided image path");
    let device = FsFileBackedDevice::new(file);
    Disk::new(device)
}

fn disk_from_fixture() -> Disk<FsFileBackedDevice> {
    let fixture_path = PathBuf::from("tests/data/drive.img");
    let mut fixture_file = File::open(&fixture_path)
        .unwrap_or_else(|_| panic!("Fixture file missing: {}", fixture_path.display()));

    let mut tmp = tempfile::tempfile().expect("Could not create temp file");
    std::io::copy(&mut fixture_file, &mut tmp).expect("Failed to copy fixture to temp file");
    tmp.seek(SeekFrom::Start(0)).unwrap();

    let device = FsFileBackedDevice::new(tmp);
    Disk::new(device)
}

pub fn disk() -> Disk<FsFileBackedDevice> {
    if let Some(path) = std::env::args().nth(1) {
        let p = PathBuf::from(path);
        return disk_from_path(&p);
    }
    disk_from_fixture()
}

pub fn pad_or_truncate_to_11_bytes(input: &str) -> [u8; 11] {
    let mut result = [b' '; 11];
    let bytes = input.as_bytes();
    let len = core::cmp::min(bytes.len(), 8);
    result[..len].copy_from_slice(&bytes[..len]);
    result
}
