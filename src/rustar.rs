
use std::io::{self, Read};

struct TarHeader {
    file_name: String,
    file_size: usize,
}

pub fn run_tar(args: &[String]) -> io::Result<()> {
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
    if file_provided {
        list_tar_files(files)?;
    } else {
        list_tar_stdin()?;
    }
    Ok(())
}

fn list_tar_files(files: &[&String]) -> io::Result<()> {
    let tarball = &files[0];
    let buffer: String = std::fs::read_to_string(tarball)?;
    get_tar_files(buffer.into_bytes())?;
    Ok(())
}

fn list_tar_stdin() -> io::Result<()> {
    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer)?;
    get_tar_files(buffer)?;
    Ok(())
}

fn get_tar_files(buffer: Vec<u8>) -> io::Result<()> {
    let _blocks = buffer.as_slice();
    // split buffer into 512-byte blocks
    let mut blocks = buffer.chunks_exact(512);
    let mut file_block_cnt = 0;
    let mut empty_block_cnt = 0;
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
            println!("{}", header.file_name);
            file_block_cnt = header.file_size / 512 + 1;
        } else {
            file_block_cnt -= 1;
        }
        empty_block_cnt = 0;
    }
    Ok(())
}

fn is_all_zero(block: &[u8]) -> bool {
    block.iter().all(|&x| x == 0)
}

fn parse_header(block: &[u8]) -> TarHeader {
    let file_name = String::from_utf8_lossy(&block[0..100]);
    let file_size =
        usize::from_str_radix(&String::from_utf8_lossy(&block[124..135]).trim(), 8).unwrap();
    TarHeader {
        file_name: file_name.to_string(),
        file_size: file_size,
    }
}
