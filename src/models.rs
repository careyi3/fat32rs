const PARTITION_BLOCK_SIZE: u32 = 512;

#[derive(Default, Clone, Copy)]
pub struct Partition {
    pub boot_flag: u8,
    pub start_chs: [u8; 3],
    pub part_type: u8,
    pub end_chs: [u8; 3],
    pub start_lba: u32,
    pub num_sectors: u32,
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

    pub fn get_partition_offset(&self) -> u64 {
        let partition_sector_offset = self.start_lba;
        let partition_offset = partition_sector_offset * PARTITION_BLOCK_SIZE;
        partition_offset as u64
    }
}

pub struct BiosParameterBlock {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sector_count: u16,
    pub num_fats: u8,
    pub total_sectors_16: u16,
    pub total_sectors_32: u32,
    pub fat_size_16: u16,
    pub fat_size_32: u32,
    pub root_cluster: u32,
    pub fs_info_sector: u16,
    pub backup_boot_sector: u16,
    pub fat_size: u32,
    pub total_sectors: u32,
    pub fat_start_sector: u16,
    pub data_start_sector: u32,
    pub root_dir_first_sector: u32,
}

impl BiosParameterBlock {
    pub fn from_bytes(bytes: [u8; 512]) -> Self {
        let bytes_per_sector = u16::from_le_bytes([bytes[11], bytes[12]]);
        let sectors_per_cluster = bytes[13];
        let reserved_sector_count = u16::from_le_bytes([bytes[14], bytes[15]]);
        let num_fats = bytes[16];
        let total_sectors_16 = u16::from_le_bytes([bytes[19], bytes[20]]);
        let total_sectors_32 = u32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]);
        let fat_size_16 = u16::from_le_bytes([bytes[22], bytes[23]]);
        let fat_size_32 = u32::from_le_bytes([bytes[36], bytes[37], bytes[38], bytes[39]]);
        let root_cluster = u32::from_le_bytes([bytes[44], bytes[45], bytes[46], bytes[47]]);
        let fs_info_sector = u16::from_le_bytes([bytes[48], bytes[49]]);
        let backup_boot_sector = u16::from_le_bytes([bytes[50], bytes[51]]);

        Self {
            bytes_per_sector,
            sectors_per_cluster,
            reserved_sector_count,
            num_fats,
            total_sectors_16,
            total_sectors_32,
            fat_size_16,
            fat_size_32,
            root_cluster,
            fs_info_sector,
            backup_boot_sector,
            fat_size: fat_size_32,
            total_sectors: total_sectors_32,
            fat_start_sector: reserved_sector_count,
            data_start_sector: reserved_sector_count as u32 + (num_fats as u32 * fat_size_32),
            root_dir_first_sector: root_cluster,
        }
    }
}
