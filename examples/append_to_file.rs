#[path = "../tests/test_helpers.rs"]
mod test_helpers;

use fat32rs::disk::Result;
use test_helpers::{disk, pad_or_truncate_to_11_bytes};

fn main() -> Result<()> {
    let mut disk = disk();
    disk.init()?;

    let mut file = disk
        .get_root_file_by_name(pad_or_truncate_to_11_bytes("LOG-1"))
        .unwrap()?;

    println!("Before append:");
    let mut content = String::new();
    for byte in disk.read_file(file)? {
        content.push(byte? as char);
    }
    println!(
        "File: {}, Size: {}, Content:\n{}",
        std::str::from_utf8(&file.name).unwrap().trim(),
        file.size,
        content
    );

    let mut random_data = Vec::with_capacity(1000);
    for i in 0..1000 {
        let ascii_char = (32 + (i % 95)) as u8;
        random_data.push(ascii_char);
    }

    disk.append_to_file(&mut file, &random_data)?;

    println!("After append:");
    let mut content = String::new();
    for byte in disk.read_file(file)? {
        content.push(byte? as char);
    }
    println!(
        "File: {}, Size: {}, Content:\n{}",
        std::str::from_utf8(&file.name).unwrap().trim(),
        file.size,
        content
    );

    Ok(())
}
