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

    assert_eq!(disk.partition.unwrap().boot_flag, 0);
    assert_eq!(disk.partition.unwrap().start_chs, [254, 255, 255]);
    assert_eq!(disk.partition.unwrap().part_type, 11);
    assert_eq!(disk.partition.unwrap().end_chs, [254, 255, 255]);
    assert_eq!(disk.partition.unwrap().start_lba, 1);
    assert_eq!(disk.partition.unwrap().num_sectors, 131071);

    // Partition 0 assertions
    assert_eq!(partitions[0].boot_flag, 0x00);
    assert_eq!(partitions[0].start_chs, [254, 255, 255]);
    assert_eq!(partitions[0].part_type, 11);
    assert_eq!(partitions[0].end_chs, [254, 255, 255]);
    assert_eq!(partitions[0].start_lba, 1);
    assert_eq!(partitions[0].num_sectors, 131071);

    // Partition 1 assertions
    assert_eq!(partitions[1].boot_flag, 0x00);
    assert_eq!(partitions[1].start_chs, [0, 0, 0]);
    assert_eq!(partitions[1].part_type, 0x00);
    assert_eq!(partitions[1].end_chs, [0, 0, 0]);
    assert_eq!(partitions[1].start_lba, 0);
    assert_eq!(partitions[1].num_sectors, 0);

    // Partition 2 assertions
    assert_eq!(partitions[2].boot_flag, 0x00);
    assert_eq!(partitions[2].start_chs, [0, 0, 0]);
    assert_eq!(partitions[2].part_type, 0x00);
    assert_eq!(partitions[2].end_chs, [0, 0, 0]);
    assert_eq!(partitions[2].start_lba, 0);
    assert_eq!(partitions[2].num_sectors, 0);

    // Partition 3 assertions
    assert_eq!(partitions[3].boot_flag, 0x00);
    assert_eq!(partitions[3].start_chs, [0, 0, 0]);
    assert_eq!(partitions[3].part_type, 0x00);
    assert_eq!(partitions[3].end_chs, [0, 0, 0]);
    assert_eq!(partitions[3].start_lba, 0);
    assert_eq!(partitions[3].num_sectors, 0);

    let bios_parameter_block = disk.bios_parameter_block.unwrap();

    assert_eq!(bios_parameter_block.bytes_per_sector, 512);
    assert_eq!(bios_parameter_block.sectors_per_cluster, 1);
    assert_eq!(bios_parameter_block.reserved_sector_count, 32);
    assert_eq!(bios_parameter_block.num_fats, 2);
    assert_eq!(bios_parameter_block.total_sectors_16, 0);
    assert_eq!(bios_parameter_block.total_sectors_32, 131070);
    assert_eq!(bios_parameter_block.fat_size_16, 0);
    assert_eq!(bios_parameter_block.fat_size_32, 1008);
    assert_eq!(bios_parameter_block.root_cluster, 2);
    assert_eq!(bios_parameter_block.fs_info_sector, 1);
    assert_eq!(bios_parameter_block.backup_boot_sector, 6);
    assert_eq!(bios_parameter_block.fat_size, 1008);
    assert_eq!(bios_parameter_block.total_sectors, 131070);
    assert_eq!(bios_parameter_block.fat_start_sector, 32);
    assert_eq!(bios_parameter_block.data_start_sector, 2048);
    assert_eq!(bios_parameter_block.root_dir_first_sector, 2);
}
