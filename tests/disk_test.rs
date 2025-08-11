mod test_helpers;

use test_helpers::disk;

use fat32rs::models::File;

#[test]
fn it_reads_and_writes_to_disk() {
    let mut disk = disk();
    let data = [42u8; 512];
    disk.write_file_block(0, data).unwrap();

    let block = disk.read_file_block(0).unwrap();
    println!("Read block starts with: {:?}", &block[..8]);

    assert_eq!(block, data);
}

#[test]
fn it_inits() {
    let mut disk = disk();
    disk.init().unwrap();

    assert_eq!(disk.reads, 2);
    assert_eq!(disk.writes, 0);

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
    assert_eq!(partitions[0].byte_offset, 512);

    // Partition 1 assertions
    assert_eq!(partitions[1].boot_flag, 0x00);
    assert_eq!(partitions[1].start_chs, [0, 0, 0]);
    assert_eq!(partitions[1].part_type, 0x00);
    assert_eq!(partitions[1].end_chs, [0, 0, 0]);
    assert_eq!(partitions[1].start_lba, 0);
    assert_eq!(partitions[1].num_sectors, 0);
    assert_eq!(partitions[1].byte_offset, 0);

    // Partition 2 assertions
    assert_eq!(partitions[2].boot_flag, 0x00);
    assert_eq!(partitions[2].start_chs, [0, 0, 0]);
    assert_eq!(partitions[2].part_type, 0x00);
    assert_eq!(partitions[2].end_chs, [0, 0, 0]);
    assert_eq!(partitions[2].start_lba, 0);
    assert_eq!(partitions[2].num_sectors, 0);
    assert_eq!(partitions[2].byte_offset, 0);

    // Partition 3 assertions
    assert_eq!(partitions[3].boot_flag, 0x00);
    assert_eq!(partitions[3].start_chs, [0, 0, 0]);
    assert_eq!(partitions[3].part_type, 0x00);
    assert_eq!(partitions[3].end_chs, [0, 0, 0]);
    assert_eq!(partitions[3].start_lba, 0);
    assert_eq!(partitions[3].num_sectors, 0);
    assert_eq!(partitions[3].byte_offset, 0);

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
    assert_eq!(bios_parameter_block.data_start_sector, 2048);
    assert_eq!(bios_parameter_block.bytes_per_cluster, 512);
    assert_eq!(bios_parameter_block.data_sector_bytes_offset, 1048576);
    assert_eq!(bios_parameter_block.fat_table_byte_offset, 16384);
}

#[test]
fn it_lists_the_root_files() {
    let mut disk = disk();
    disk.init().unwrap();
    let filelist = disk.list_root_files();
    let mut count = 0;
    let mut count_non_empty = 0;
    for files in filelist {
        for file in files {
            count += 1;
            if file.name != [0; 11] {
                count_non_empty += 1;
            }
        }
    }
    assert_eq!(disk.reads, 4);
    assert_eq!(disk.writes, 0);
    assert_eq!(count, 16);
    assert_eq!(count_non_empty, 9);
}

#[test]
fn it_reads_files_in_chunks() {
    let mut disk = disk();
    disk.init().unwrap();
    let filelist = disk.list_root_files();
    let mut to_read: File = File::default();
    for files in filelist {
        for file in files {
            if std::str::from_utf8(&file.name).unwrap().trim() == "LOG-1" {
                to_read = file;
            }
        }
    }
    assert_eq!(disk.reads, 4);
    assert_eq!(disk.writes, 0);

    let mut content = String::new();
    for chunk in disk.read_file_in_chunks(to_read) {
        content += std::str::from_utf8(&chunk.0[..(to_read.size % 512) as usize]).unwrap();
    }
    assert_eq!(disk.reads, 6);
    assert_eq!(disk.writes, 0);

    assert_eq!(content, "log line 1\n");
}

#[test]
fn it_reads_larger_files_in_chunks() {
    let mut disk = disk();
    disk.init().unwrap();
    let filelist = disk.list_root_files();
    let mut to_read: File = File::default();
    for files in filelist {
        for file in files {
            if std::str::from_utf8(&file.name).unwrap().trim() == "LOG-2" {
                to_read = file;
            }
        }
    }
    assert_eq!(disk.reads, 4);
    assert_eq!(disk.writes, 0);

    let mut content = String::new();
    let mut read = 0;
    for chunk in disk.read_file_in_chunks(to_read) {
        read += 512;
        if read > to_read.size {
            content += std::str::from_utf8(&chunk.0[..(to_read.size % 512) as usize]).unwrap();
        } else {
            content += std::str::from_utf8(&chunk.0).unwrap();
        }
    }
    assert_eq!(disk.reads, 208);
    assert_eq!(disk.writes, 0);

    assert_eq!(content.len(), 52117);
    assert_eq!(to_read.size, 52117);
}
