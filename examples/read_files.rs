#[path = "../tests/test_helpers.rs"]
mod test_helpers;

use test_helpers::disk;

fn main() -> std::io::Result<()> {
    let mut disk = disk();

    disk.init().unwrap();
    let mut to_read = vec![];
    let filelist = disk.list_root_files().unwrap();
    for result in filelist {
        let files = result.unwrap();
        for file in files {
            if file.attributes == 32 && file.size > 0 {
                to_read.push(file);
            }
        }
    }

    for file in to_read {
        let mut content = String::new();
        let mut read = 0;
        for result in disk.read_file_in_chunks(file).unwrap() {
            let chunk = result.unwrap();
            read += 512;
            if read > file.size {
                content += std::str::from_utf8(&chunk.0[..(file.size % 512) as usize]).unwrap();
            } else {
                content += std::str::from_utf8(&chunk.0).unwrap();
            }
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
