pub struct TarHeader {
    pub file_name: String,
    pub file_size: usize,
}

pub struct ArchivedFile<'a> {
    pub header: TarHeader,          // contains file name and size
    pub data_blocks: Vec<&'a [u8]>, // contains file data
}
