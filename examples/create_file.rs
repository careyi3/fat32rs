#[path = "../tests/test_helpers.rs"]
mod test_helpers;

use fat32rs::disk::Result;
use test_helpers::{disk, pad_or_truncate_to_11_bytes};

fn main() -> Result<()> {
    let mut disk = disk();
    disk.init()?;

    println!("Before create:");
    let files = disk.list_root_files().unwrap();
    for file in files {
        println!(
            "Name: {}, Size: {}",
            std::str::from_utf8(&file?.name).unwrap().trim(),
            file?.size,
        )
    }

    disk.create_file(pad_or_truncate_to_11_bytes("new"))?;

    println!("After create:");
    let files = disk.list_root_files().unwrap();
    for file in files {
        println!(
            "Name: {}, Size: {}",
            std::str::from_utf8(&file?.name).unwrap().trim(),
            file?.size,
        )
    }

    Ok(())
}
