use super::models::{ArchivedFile, TarHeader};

// a file is a header + data blocks
pub fn parse_to_file_blocks(buffer: &Vec<u8>) -> Vec<ArchivedFile> {
    let blocks = buffer.chunks_exact(512);
    let mut files: Vec<ArchivedFile> = Vec::new();
    let mut file_block_cnt = 0;
    let mut empty_block_cnt = 0;
    let mut header: Option<TarHeader> = None;
    let mut data: Vec<&[u8]> = Vec::new();

    for block in blocks {
        if empty_block_cnt == 2 {
            break;
        }
        if empty_block_cnt < 2 && is_all_zero(block) {
            empty_block_cnt += 1;
            continue;
        }
        if file_block_cnt == 0 {
            if header.is_some() {
                files.push(ArchivedFile {
                    header: header.unwrap(),
                    data_blocks: data,
                });
            }
            let temp_header = parse_header(block);
            file_block_cnt = temp_header.file_size / 512 + 1;
            header = Some(temp_header);
            data = Vec::new();
        } else {
            file_block_cnt -= 1;
            data.push(block);
        }
        empty_block_cnt = 0;
    }
    // push the last file
    files.push(ArchivedFile {
        header: header.unwrap(),
        data_blocks: data,
    });
    files
}

fn is_all_zero(block: &[u8]) -> bool {
    block.iter().all(|&x| x == 0)
}

fn parse_header(block: &[u8]) -> TarHeader {
    let file_name = String::from_utf8_lossy(&block[0..100])
        .trim_matches('\0')
        .to_string();
    let file_size =
        usize::from_str_radix(&String::from_utf8_lossy(&block[124..135]).trim(), 8).unwrap();
    TarHeader {
        file_name,
        file_size,
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FILE: &str = "test.txt";

    #[test]
    fn test_parse_to_file_blocks_success() {
        let buffer = create_tar_file_buffer();
        let files = parse_to_file_blocks(&buffer);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].header.file_name, TEST_FILE);
    } 

    fn create_tar_file_buffer() -> Vec<u8> {
        let mut header = tar::Header::new_gnu();
        header.set_path(TEST_FILE).unwrap();
        header.set_size(4);
        header.set_cksum();

        let data: &[u8] = &[1, 2, 3];

        let mut tar_builder = tar::Builder::new(Vec::new());
        tar_builder.append(&header, data).unwrap();
        tar_builder.finish().unwrap();
        tar_builder.into_inner().unwrap()
    }
}
