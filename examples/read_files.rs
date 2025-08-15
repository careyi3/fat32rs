#[path = "../tests/test_helpers.rs"]
mod test_helpers;

use fat32rs::disk::Result;
use test_helpers::disk;

fn main() -> Result<()> {
    let mut disk = disk();

    disk.init().unwrap();
    let mut to_read = vec![];
    let files = disk.list_root_files().unwrap();
    for result in files {
        let file = result.unwrap();
        to_read.push(file);
    }

    for file in to_read {
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
    }

    Ok(())
}
