use crate::models::Partition;

pub type Result<T> = core::result::Result<T, IOError>;

#[derive(Debug, Clone, Copy)]
pub enum IOError {
    ReadError,
    WriteError,
}

pub trait BlockIO {
    fn read_block(&mut self, block_index: u64) -> Result<[u8; 512]>;
    fn write_block(&mut self, block_index: u64, data: [u8; 512]) -> Result<()>;
}

pub struct Disk<T: BlockIO> {
    io: T,
    pub partitions: [Partition; 4],
    pub partition: Option<Partition>,
}

impl<T: BlockIO> Disk<T> {
    pub fn new(io: T) -> Self {
        Self {
            io,
            partitions: Default::default(),
            partition: None,
        }
    }

    pub fn read_file_block(&mut self, block_index: u64) -> Result<[u8; 512]> {
        self.io.read_block(block_index)
    }

    pub fn write_file_block(&mut self, block_index: u64, data: [u8; 512]) -> Result<()> {
        self.io.write_block(block_index, data)
    }

    pub fn list_files(&self) -> Result<&'static [&'static str]> {
        static FILES: [&str; 2] = ["file1.txt", "file2.txt"];
        Ok(&FILES)
    }

    pub fn init(&mut self) -> Result<()> {
        let data = self.read_file_block(0)?;
        self.partitions = Partition::from_bytes(data);
        self.partition = Some(self.partitions[0]);
        Ok(())
    }
}
