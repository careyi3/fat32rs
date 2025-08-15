mod test_helpers;

use test_helpers::{disk, pad_or_truncate_to_11_bytes};

use fat32rs::disk::{Error, Result};

#[test]
fn it_inits() -> Result<()> {
    let mut disk = disk();
    disk.init()?;

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
    Ok(())
}

#[test]
fn it_lists_the_root_files() -> Result<()> {
    let mut disk = disk();
    disk.init()?;
    let files = disk.list_root_files().unwrap();
    let mut count = 0;
    let mut count_non_empty = 0;
    for result in files {
        let file = result.unwrap();
        count += 1;
        if file.name != [0; 11] {
            count_non_empty += 1;
        }
    }
    assert_eq!(disk.reads, 2);
    assert_eq!(disk.writes, 0);
    assert_eq!(count, 2);
    assert_eq!(count_non_empty, 2);
    Ok(())
}

#[test]
fn it_reads_files() -> Result<()> {
    let mut disk = disk();
    disk.init()?;

    let to_read = disk
        .get_root_file_by_name(pad_or_truncate_to_11_bytes("LOG-1"))
        .unwrap()?;
    assert_eq!(disk.reads, 2);
    assert_eq!(disk.writes, 0);

    let mut content = String::new();
    for byte in disk.read_file(to_read)? {
        content.push(byte? as char);
    }
    assert_eq!(disk.reads, 2);
    assert_eq!(disk.writes, 0);

    assert_eq!(content, "log line 1\n");
    Ok(())
}

#[test]
fn it_reads_larger_files() -> Result<()> {
    let mut disk = disk();
    disk.init()?;

    let to_read = disk
        .get_root_file_by_name(pad_or_truncate_to_11_bytes("LOG-2"))
        .unwrap()?;
    assert_eq!(disk.reads, 2);
    assert_eq!(disk.writes, 0);

    let mut content = String::new();
    for byte in disk.read_file(to_read)? {
        content.push(byte? as char);
    }

    assert_eq!(disk.reads, 204);
    assert_eq!(disk.writes, 0);

    assert_eq!(content.len(), 52117);
    assert_eq!(to_read.size, 52117);
    Ok(())
}

#[test]
fn it_can_append_to_file() -> Result<()> {
    let mut disk = disk();
    disk.init()?;

    let mut to_test = disk
        .get_root_file_by_name(pad_or_truncate_to_11_bytes("LOG-1"))
        .unwrap()?;
    assert_eq!(disk.reads, 2);
    assert_eq!(disk.writes, 0);

    let mut content = String::new();
    for byte in disk.read_file(to_test)? {
        content.push(byte? as char);
    }
    assert_eq!(disk.reads, 2);
    assert_eq!(disk.writes, 0);

    assert_eq!(content, "log line 1\n");

    disk.append_to_file(&mut to_test, b"new data")?;
    assert_eq!(disk.reads, 3);
    assert_eq!(disk.writes, 2);

    let to_test = disk
        .get_root_file_by_name(pad_or_truncate_to_11_bytes("LOG-1"))
        .unwrap()?;

    let mut content = String::new();
    for byte in disk.read_file(to_test)? {
        content.push(byte? as char);
    }
    assert_eq!(disk.reads, 2);
    assert_eq!(disk.writes, 0);

    assert_eq!(content, "log line 1\nnew data");
    assert_eq!(to_test.size, 19);
    Ok(())
}

#[test]
fn it_can_append_lots_of_data_to_file() -> Result<()> {
    let mut disk = disk();
    disk.init()?;

    let mut to_test = disk
        .get_root_file_by_name(pad_or_truncate_to_11_bytes("LOG-1"))
        .unwrap()?;
    assert_eq!(disk.reads, 2);
    assert_eq!(disk.writes, 0);

    let mut content = String::new();
    for byte in disk.read_file(to_test)? {
        content.push(byte? as char);
    }
    assert_eq!(disk.reads, 2);
    assert_eq!(disk.writes, 0);

    assert_eq!(content, "log line 1\n");

    let mut random_data = Vec::with_capacity(1000);
    for i in 0..1000 {
        let ascii_char = (32 + (i % 95)) as u8;
        random_data.push(ascii_char);
    }
    let expected_appended_data = String::from_utf8(random_data.clone()).unwrap();

    disk.append_to_file(&mut to_test, &random_data)?;
    assert_eq!(disk.reads, 11);
    assert_eq!(disk.writes, 5);

    let to_test = disk
        .get_root_file_by_name(pad_or_truncate_to_11_bytes("LOG-1"))
        .unwrap()?;

    let mut content = String::new();
    for byte in disk.read_file(to_test)? {
        content.push(byte? as char);
    }
    assert_eq!(disk.reads, 4);
    assert_eq!(disk.writes, 0);

    let expected_content = format!("log line 1\n{}", expected_appended_data);
    assert_eq!(content, expected_content);
    assert_eq!(to_test.size, 1011);
    Ok(())
}

#[test]
fn it_can_create_a_new_file() -> Result<()> {
    let mut disk = disk();
    disk.init()?;

    let name = pad_or_truncate_to_11_bytes("new file");

    disk.create_file(name)?;

    let to_test = disk
        .get_root_file_by_name(pad_or_truncate_to_11_bytes("new file"))
        .unwrap()?;

    assert_eq!(
        std::str::from_utf8(&to_test.name).unwrap().trim(),
        "new file"
    );
    Ok(())
}

#[test]
fn it_can_append_to_newly_created_files() -> Result<()> {
    let mut disk = disk();
    disk.init()?;

    let name = pad_or_truncate_to_11_bytes("new file");

    disk.create_file(name)?;

    let mut to_test = disk
        .get_root_file_by_name(pad_or_truncate_to_11_bytes("new file"))
        .unwrap()?;

    assert_eq!(
        std::str::from_utf8(&to_test.name).unwrap().trim(),
        "new file"
    );

    disk.append_to_file(&mut to_test, b"new data")?;

    let mut content = String::new();
    for byte in disk.read_file(to_test)? {
        content.push(byte? as char);
    }
    assert_eq!(disk.reads, 2);
    assert_eq!(disk.writes, 0);

    assert_eq!(content, "new data");

    Ok(())
}

#[test]
fn it_can_create_many_files() -> Result<()> {
    let mut disk = disk();
    disk.init()?;

    for i in 0..100 {
        let name = pad_or_truncate_to_11_bytes(format!("test-{}", i).as_str());

        disk.create_file(name)?;
    }

    let mut count = 0;
    for _ in disk.list_root_files()? {
        count += 1;
    }

    assert_eq!(count, 102);

    Ok(())
}

#[test]
fn it_can_create_and_write_lots_of_data() -> Result<()> {
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
        size += file?.size;
        count += 1;
    }

    assert_eq!(count, 52);
    assert_eq!(size, 3302128);

    Ok(())
}

#[test]
fn it_errors_when_the_disk_is_full() -> Result<()> {
    let mut disk = disk();
    disk.init()?;

    for i in 0..5000 {
        let name = pad_or_truncate_to_11_bytes(format!("test-{}", i).as_str());
        let file_result = disk.create_file(name);

        if file_result.is_err() {
            assert_eq!(file_result.err().unwrap(), Error::DiskFullError);
            return Ok(());
        }

        let mut file = file_result?;

        let mut random_data = Vec::with_capacity(6500);
        for i in 0..6500 {
            let ascii_char = (32 + (i % 95)) as u8;
            random_data.push(ascii_char);
        }

        for _ in 0..10 {
            let file_result = disk.append_to_file(&mut file, &random_data);
            if file_result.is_err() {
                assert_eq!(file_result.err().unwrap(), Error::DiskFullError);
                return Ok(());
            }
        }
    }

    Ok(())
}
