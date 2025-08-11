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

    pub fn read_file_block(&mut self, byte_offset: u64) -> Result<[u8; 512]> {
        self.reads += 1;
        self.io.read_block(byte_offset)
    }

    pub fn write_file_block(&mut self, byte_offset: u64, data: [u8; 512]) -> Result<()> {
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

    pub fn read_file_in_chunks(&mut self, file: File) -> FilePointer<T> {
        make_file_pointer(file, self)
    }

    pub fn list_root_files(&mut self) -> FileList<T> {
        make_file_list(self)
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
}

fn first_largest_non_zero_partition(partitions: &[Partition; 4]) -> Option<Partition> {
    partitions
        .iter()
        .filter(|p| p.num_sectors > 0)
        .copied()
        .max_by_key(|p| p.num_sectors)
}
