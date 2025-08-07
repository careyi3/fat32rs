mod test_helpers;

use test_helpers::disk;

#[test]
fn it_reads_and_writes_to_disk() {
    let mut disk = disk();
    let data = [42u8; 512];
    disk.write_file_block(0, data).unwrap();

    let block = disk.read_file_block(0).unwrap();
    println!("Read block starts with: {:?}", &block[..8]);

    let files = disk.list_files().unwrap();
    println!("Files on disk: {:?}", files);
    assert_eq!(block, data);
}

#[test]
fn it_inits() {
    let mut disk = disk();
    disk.init().unwrap();
    let partitions = disk.partitions;
    assert_eq!(partitions.len(), 4);
}
