use crate::models::{BiosParameterBlock, File, Partition};

pub type Result<T> = core::result::Result<T, IOError>;

const LOGICAL_BLOCK_SIZE: u64 = 512;
const MAX_FILE_SIZE: u64 = 4294967296;

#[derive(Debug, Clone, Copy)]
pub enum IOError {
    ReadError,
    WriteError,
}

pub trait BlockIO {
    fn read_block(&mut self, byte_offset: u64) -> Result<[u8; 512]>;
    fn write_block(&mut self, byte_offset: u64, data: [u8; 512]) -> Result<()>;
}

pub struct Disk<T: BlockIO> {
    io: T,
    pub partitions: [Partition; 4],
    pub partition: Option<Partition>,
    pub bios_parameter_block: Option<BiosParameterBlock>,
    pub reads: u32,
    pub writes: u32,
}

pub struct FilePointer<'a, T: BlockIO> {
    pub file: File,
    pub cluster: u64,
    pub sector: u64,
    pub sector_count: u64,
    pub disk: &'a mut Disk<T>,
}

impl<'a, T: BlockIO> Iterator for FilePointer<'a, T> {
    type Item = ([u8; 512], u64);
    fn next(&mut self) -> Option<Self::Item> {
        let partition_offset = self.disk.partition.unwrap().byte_offset;
        let bios_parameter_block = self.disk.bios_parameter_block.as_ref().unwrap();
        let data_sector_byte_offset = bios_parameter_block.data_sector_bytes_offset;
        let bytes_per_cluster = bios_parameter_block.bytes_per_cluster;
        let root_dir_first_cluster = bios_parameter_block.root_cluster;
        let sectors_per_cluster = bios_parameter_block.sectors_per_cluster;

        let sector_to_read = self.sector % sectors_per_cluster;

        if self.sector == self.sector_count || self.cluster >= 0x0FFFFFF8 {
            None
        } else {
            let offset = data_sector_byte_offset
                + ((self.cluster - root_dir_first_cluster) * bytes_per_cluster);

            let data = self
                .disk
                .read_file_block(partition_offset + offset + (sector_to_read * LOGICAL_BLOCK_SIZE))
                .unwrap();

            self.sector += 1;
            if self.sector % sectors_per_cluster == 0 {
                self.cluster = self.disk.get_next_cluster(self.cluster).unwrap();
            }

            Some((data, offset + (sector_to_read * LOGICAL_BLOCK_SIZE)))
        }
    }
}

fn make_file_pointer<T: BlockIO>(file: File, disk: &mut Disk<T>) -> FilePointer<T> {
    let cluster = file.start_cluster as u64;

    let mut sector_count = file.size / LOGICAL_BLOCK_SIZE;
    if file.size % LOGICAL_BLOCK_SIZE != 0 {
        sector_count += 1;
    }

    FilePointer {
        file,
        cluster,
        sector: 0,
        sector_count,
        disk,
    }
}

pub struct FileList<'a, T: BlockIO> {
    pub file_pointer: FilePointer<'a, T>,
}

impl<'a, T: BlockIO> Iterator for FileList<'a, T> {
    type Item = [File; 16];
    fn next(&mut self) -> Option<Self::Item> {
        let next_block = self.file_pointer.next();
        if next_block.is_none() {
            return None;
        } else {
            let (data, offset) = next_block.unwrap();
            Some(File::from_bytes(data, offset))
        }
    }
}

fn make_file_list<T: BlockIO>(disk: &mut Disk<T>) -> FileList<T> {
    let root_dir_first_cluster = disk.bios_parameter_block.as_ref().unwrap().root_cluster;
    let file = File {
        name: [0; 11],
        attributes: 0x20,
        start_cluster: root_dir_first_cluster,
        size: MAX_FILE_SIZE,
        is_lfn: false,
        byte_offset: 0,
    };
    let file_pointer = make_file_pointer(file, disk);
    FileList { file_pointer }
}

impl<T: BlockIO> Disk<T> {
    pub fn new(io: T) -> Self {
        Self {
            io,
            partitions: Default::default(),
            partition: None,
            bios_parameter_block: None,
            reads: 0,
            writes: 0,
        }
    }

    fn read_file_block(&mut self, byte_offset: u64) -> Result<[u8; 512]> {
        self.reads += 1;
        self.io.read_block(byte_offset)
    }

    fn write_file_block(&mut self, byte_offset: u64, data: [u8; 512]) -> Result<()> {
        self.writes += 1;
        self.io.write_block(byte_offset, data)
    }

    fn get_next_cluster(&mut self, cluster: u64) -> Result<u64> {
        let partition_offset = self.partition.unwrap().byte_offset;
        let bios_parameter_block = self.bios_parameter_block.as_ref().unwrap();
        let offset = bios_parameter_block.fat_table_byte_offset;
        let bytes_per_sector = bios_parameter_block.bytes_per_sector;

        let cluster_byte_start = cluster * 4;
        let sector_num = cluster_byte_start / bytes_per_sector;

        let data =
            self.read_file_block(partition_offset + offset + (sector_num * LOGICAL_BLOCK_SIZE))?;

        let start = (cluster_byte_start % LOGICAL_BLOCK_SIZE) as usize;
        let next_cluster = u32::from_le_bytes(data[start..start + 4].try_into().unwrap());
        Ok((next_cluster & 0x0FFFFFFF) as u64)
    }

    fn get_files_last_cluster(&mut self, file: &File) -> u64 {
        let mut cluster = file.start_cluster as u64;
        while let Ok(next_cluster) = self.get_next_cluster(cluster) {
            if next_cluster >= 0x0FFFFFF8 {
                break;
            }
            cluster = next_cluster;
        }
        cluster
    }

    fn write_to_last_cluster(&mut self, file: &File, data: &[u8], written: &mut u64) -> Result<()> {
        let partition_offset = self.partition.unwrap().byte_offset;
        let bytes_per_cluster = self
            .bios_parameter_block
            .as_ref()
            .unwrap()
            .bytes_per_cluster;
        let data_sector_bytes_offset = self
            .bios_parameter_block
            .as_ref()
            .unwrap()
            .data_sector_bytes_offset;
        let root_dir_first_cluster = self.bios_parameter_block.as_ref().unwrap().root_cluster;
        let file_size = file.size;
        let mut free_bytes_in_cluster = bytes_per_cluster - (file_size % bytes_per_cluster);
        let last_cluster = self.get_files_last_cluster(file);

        let cluster_used_bytes = file_size % bytes_per_cluster;
        let mut used_sectors = cluster_used_bytes / LOGICAL_BLOCK_SIZE;
        let mut eof_index = cluster_used_bytes % LOGICAL_BLOCK_SIZE;
        let offset = data_sector_bytes_offset
            + ((last_cluster - root_dir_first_cluster) * bytes_per_cluster);

        let mut block =
            self.read_file_block(partition_offset + offset + (used_sectors * LOGICAL_BLOCK_SIZE))?;

        while free_bytes_in_cluster > 0 {
            if *written == data.len() as u64 {
                break;
            }

            let d = data[*written as usize];
            block[eof_index as usize] = d;
            *written += 1;
            free_bytes_in_cluster -= 1;
            eof_index += 1;
            if eof_index == LOGICAL_BLOCK_SIZE {
                self.write_file_block(
                    partition_offset + offset + (used_sectors * LOGICAL_BLOCK_SIZE),
                    block,
                )?;
                used_sectors += 1;
                eof_index = 0;
                block = self.read_file_block(
                    partition_offset + offset + (used_sectors * LOGICAL_BLOCK_SIZE),
                )?;
            }
        }

        self.write_file_block(
            partition_offset + offset + (used_sectors * LOGICAL_BLOCK_SIZE),
            block,
        )?;

        Ok(())
    }

    fn find_next_empty_fat_entry(&mut self, cluster: u64) -> ([u8; 512], u64, u64) {
        let partition_offset = self.partition.unwrap().byte_offset;
        let fat_table_byte_offset = self
            .bios_parameter_block
            .as_ref()
            .unwrap()
            .fat_table_byte_offset;
        let fat_sectors = self.bios_parameter_block.as_ref().unwrap().fat_size_32;

        let mut idx: Option<u64> = None;
        let start_sector = (cluster * 4) / LOGICAL_BLOCK_SIZE;
        let mut sector_num = start_sector;
        let mut data = [0u8; 512];
        let mut start_id = (cluster * 4) % LOGICAL_BLOCK_SIZE;

        'outer: while idx.is_none() {
            data = self
                .read_file_block(
                    partition_offset + fat_table_byte_offset + (sector_num * LOGICAL_BLOCK_SIZE),
                )
                .unwrap();
            for i in (start_id as usize..data.len()).step_by(4) {
                let start_lba_arr: [u8; 4] = data[i..i + 4].try_into().unwrap();
                let content: u64 = u32::from_le_bytes(start_lba_arr) as u64 & 0x0FFFFFFF;
                if content == 0x00000000 {
                    idx = Some(i as u64);
                    break 'outer;
                }
                start_id = 0;
                sector_num += 1;
                if sector_num == fat_sectors {
                    sector_num = 0;
                }
                if start_sector == sector_num {
                    //TODO: Handle this properly
                    panic!("Can't allocate any more memory")
                }
            }
        }

        return (data, sector_num, idx.unwrap());
    }

    fn write_fat_entry(
        &mut self,
        mut data: [u8; 512],
        sector_num: u64,
        idx: u64,
        entry: [u8; 4],
    ) -> Result<()> {
        let partition_offset = self.partition.unwrap().byte_offset;
        let fat_table_byte_offset = self
            .bios_parameter_block
            .as_ref()
            .unwrap()
            .fat_table_byte_offset;

        data[idx as usize] = entry[0];
        data[idx as usize + 1] = entry[1];
        data[idx as usize + 2] = entry[2];
        data[idx as usize + 3] = entry[3];

        self.write_file_block(
            partition_offset + fat_table_byte_offset + (sector_num * LOGICAL_BLOCK_SIZE),
            data,
        )
    }

    fn get_fat_block_for_cluster(&mut self, cluster: u64) -> ([u8; 512], u64, u64) {
        let partition_offset = self.partition.unwrap().byte_offset;
        let fat_table_byte_offset = self
            .bios_parameter_block
            .as_ref()
            .unwrap()
            .fat_table_byte_offset;

        let idx = (cluster * 4) % LOGICAL_BLOCK_SIZE;
        let sector_num = (cluster * 4) / LOGICAL_BLOCK_SIZE;

        let data = self
            .read_file_block(
                partition_offset + fat_table_byte_offset + (sector_num * LOGICAL_BLOCK_SIZE),
            )
            .unwrap();

        return (data, sector_num, idx);
    }

    fn allocate_next_free_cluster(&mut self, last_cluster: u64) -> Result<()> {
        let (data, sector_num, idx) = self.find_next_empty_fat_entry(last_cluster);
        let entry: [u8; 4] = ((0x0FFFFFF8 & 0x0FFFFFFF) as u32).to_le_bytes();
        self.write_fat_entry(data, sector_num, idx, entry)?;

        let new_cluster = ((sector_num * LOGICAL_BLOCK_SIZE) + idx) / 4;
        let (data, sector_num, idx) = self.get_fat_block_for_cluster(last_cluster);
        let entry: [u8; 4] = ((new_cluster & 0x0FFFFFFF) as u32).to_le_bytes();
        self.write_fat_entry(data, sector_num, idx, entry)?;

        Ok(())
    }

    fn append_to_file_with_update_file_size(
        &mut self,
        file: &mut File,
        data: &[u8],
        update_file_size: bool,
    ) -> Result<File> {
        let mut written: u64 = 0;
        while written < (data.len() - 1) as u64 {
            self.write_to_last_cluster(file, data, &mut written)?;
            file.size += written;
            if written < (data.len() - 1) as u64 {
                let last_cluster = self.get_files_last_cluster(file);
                self.allocate_next_free_cluster(last_cluster)?;
            }
        }
        if update_file_size {
            Ok(*file) // TODO: Implement actual file size update logic.
        } else {
            Ok(*file)
        }
    }

    pub fn init(&mut self) -> Result<()> {
        self.reads = 0;
        self.writes = 0;
        let partition_data = self.read_file_block(0)?;
        self.partitions = Partition::from_bytes(partition_data);
        self.partition = first_largest_non_zero_partition(&self.partitions);

        let offset = self.partition.unwrap().byte_offset;
        let bios_parameter_block_data = self.read_file_block(offset)?;
        self.bios_parameter_block = Some(BiosParameterBlock::from_bytes(bios_parameter_block_data));
        Ok(())
    }

    pub fn list_root_files(&mut self) -> FileList<T> {
        self.reads = 0;
        self.writes = 0;
        make_file_list(self)
    }

    pub fn read_file_in_chunks(&mut self, file: File) -> FilePointer<T> {
        self.reads = 0;
        self.writes = 0;
        make_file_pointer(file, self)
    }

    pub fn append_to_file(&mut self, file: &mut File, data: &[u8]) -> Result<File> {
        self.reads = 0;
        self.writes = 0;
        self.append_to_file_with_update_file_size(file, data, true)
    }
}

fn first_largest_non_zero_partition(partitions: &[Partition; 4]) -> Option<Partition> {
    partitions
        .iter()
        .filter(|p| p.num_sectors > 0)
        .copied()
        .max_by_key(|p| p.num_sectors)
}
