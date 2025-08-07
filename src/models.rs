#[derive(Default, Clone, Copy)]
pub struct Partition {
    boot_flag: u8,
    start_chs: [u8; 3],
    part_type: u8,
    end_chs: [u8; 3],
    start_lba: u32,
    num_sectors: u32,
}

impl Partition {
    pub fn from_bytes(bytes: [u8; 512]) -> [Partition; 4] {
        let mut partitions: [Partition; 4] = [Partition::default(); 4];
        for i in 0..4 {
            let start_idx = 446 + (i * 16);

            let boot_flag = bytes[start_idx];
            let start_chs: [u8; 3] = bytes[(start_idx + 1)..(start_idx + 4)].try_into().unwrap();
            let part_type = bytes[start_idx + 4];
            let end_chs: [u8; 3] = bytes[(start_idx + 5)..(start_idx + 8)].try_into().unwrap();
            let start_lba_arr: [u8; 4] =
                bytes[(start_idx + 8)..(start_idx + 12)].try_into().unwrap();
            let start_lba: u32 = u32::from_le_bytes(start_lba_arr);
            let num_sectors_arr: [u8; 4] = bytes[(start_idx + 12)..(start_idx + 16)]
                .try_into()
                .unwrap();
            let num_sectors: u32 = u32::from_le_bytes(num_sectors_arr);
            let partition = Partition {
                boot_flag,
                start_chs,
                part_type,
                end_chs,
                start_lba,
                num_sectors,
            };
            partitions[i] = partition;
        }
        return partitions;
    }
}
