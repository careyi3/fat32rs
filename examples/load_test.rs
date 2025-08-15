#[path = "common/mod.rs"]
mod common;

use common::{disk, pad_or_truncate_to_11_bytes};
use fat32rs::disk::Result;

fn main() -> Result<()> {
    let mut disk = disk();
    disk.init()?;

    for i in 0..50 {
        let name = pad_or_truncate_to_11_bytes(format!("test-{}", i).as_str());
        let mut file = disk.create_file(name)?;
        let mut random_data = Vec::with_capacity(6500);
        for i in 0..6500 {
            let ascii_char = (32 + (i % 95)) as u8;
            random_data.push(ascii_char);
        }

        for _ in 0..10 {
            disk.append_to_file(&mut file, &random_data)?;
        }
    }

    let mut count = 0;
    let mut size = 0;
    for file in disk.list_root_files()? {
        if file?.name.starts_with(b"test") {
            size += file?.size;
            count += 1;
        }
    }

    println!("Created {} file, wrote {} bytes", count, size);

    Ok(())
}
