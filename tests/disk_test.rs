mod test_helpers;

use test_helpers::disk;

#[test]
fn it_runs() {
    let mut disk = disk();
    let data = [42u8; 512];
    disk.write_file_block(0, data).unwrap();

    let block = disk.read_file_block(0).unwrap();
    println!("Read block starts with: {:?}", &block[..8]);

    let files = disk.list_files().unwrap();
    println!("Files on disk: {:?}", files);
    assert_eq!(block, data);
}
