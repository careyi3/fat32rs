use crate::models::{BiosParameterBlock, Partition};

pub type Result<T> = core::result::Result<T, IOError>;

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
}

impl<T: BlockIO> Disk<T> {
    pub fn new(io: T) -> Self {
        Self {
            io,
            partitions: Default::default(),
            partition: None,
            bios_parameter_block: None,
        }
    }

    pub fn read_file_block(&mut self, byte_offset: u64) -> Result<[u8; 512]> {
        self.io.read_block(byte_offset)
    }

    pub fn write_file_block(&mut self, byte_offset: u64, data: [u8; 512]) -> Result<()> {
        self.io.write_block(byte_offset, data)
    }

    pub fn list_files(&self) -> Result<&'static [&'static str]> {
        static FILES: [&str; 2] = ["file1.txt", "file2.txt"];
        Ok(&FILES)
    }

    pub fn init(&mut self) -> Result<()> {
        let partition_data = self.read_file_block(0)?;
        self.partitions = Partition::from_bytes(partition_data);
        self.partition = first_largest_non_zero_partition(&self.partitions);

        let offset = self.partition.unwrap().get_partition_offset();
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
