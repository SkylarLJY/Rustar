use io::Result;
use std::io::{self, Read};

struct TarHeader {
    file_name: String,
    file_size: usize,
}

pub fn run_tar(args: &[String]) -> Result<()> {
    let mut flags = Vec::new();
    let mut files = Vec::new();
    for arg in args {
        if arg.starts_with('-') {
            for c in arg.chars().skip(1) {
                flags.push(c);
            }
        } else {
            files.push(arg);
        }
    }

    if flags.contains(&'t') {
        list_tar(files.as_slice(), flags.contains(&'f'))?;
    }

    Ok(())
}

fn list_tar(files: &[&String], file_provided: bool) -> io::Result<()> {
    let file_names: Vec<String>;
    if file_provided {
        file_names = list_tar_files(files)?;
    } else {
        file_names = list_tar_stdin()?;
    }
    for file_name in file_names {
        println!("{}", file_name);
    }
    Ok(())
}

fn list_tar_files(files: &[&String]) -> Result<Vec<String>> {
    let tarball = &files[0];
    let buffer: String = std::fs::read_to_string(tarball)?;
    Ok(get_tar_files(buffer.into_bytes()))
}

fn list_tar_stdin() -> Result<Vec<String>> {
    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer)?;
    Ok(get_tar_files(buffer))
}

fn get_tar_files(buffer: Vec<u8>) -> Vec<String> {
    let _blocks = buffer.as_slice();
    // split buffer into 512-byte blocks
    let mut blocks = buffer.chunks_exact(512);
    let mut file_block_cnt = 0;
    let mut empty_block_cnt = 0;
    let mut files = Vec::new();
    while let Some(block) = blocks.next() {
        if empty_block_cnt == 2 {
            break;
        }
        if empty_block_cnt < 2 && is_all_zero(block) {
            empty_block_cnt += 1;
            continue;
        }
        if file_block_cnt == 0 {
            let header = parse_header(block);
            files.push(header.file_name);
            file_block_cnt = header.file_size / 512 + 1;
        } else {
            file_block_cnt -= 1;
        }
        empty_block_cnt = 0;
    }
    files
}

fn is_all_zero(block: &[u8]) -> bool {
    block.iter().all(|&x| x == 0)
}

fn parse_header(block: &[u8]) -> TarHeader {
    let file_name = String::from_utf8_lossy(&block[0..100]);
    let name_trimed = file_name.trim_matches('\0');
    let file_size =
        usize::from_str_radix(&String::from_utf8_lossy(&block[124..135]).trim(), 8).unwrap();
    TarHeader {
        file_name: name_trimed.to_string(),
        file_size: file_size,
    }
}

#[cfg(test)]
mod tests {

    use io::BufReader;

    use super::*;

    const TEST_FILE: &str = "test.txt";

    #[test]
    fn test_get_tar_files() {
        let buffer = create_tar_file_buffer();
        let expected = vec![TEST_FILE];
        let result = get_tar_files(buffer);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_list_tar_files() {
        let tar_file = "files.tar".to_string();
        let files = [&tar_file];
        assert!(list_tar_files(&files).is_ok());
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
