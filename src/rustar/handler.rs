use std::io::{self, Read, Result, Write};
use super::parser::parse_to_file_blocks;

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
    // get input into a buffer
    let mut buffer = Vec::new();
    if flags.contains(&'f') {
        if files.is_empty() {
            eprintln!("No file specified.");
            return Ok(());
        }
        buffer = std::fs::read_to_string(&files[0])?.into();
    } else {
        io::stdin().read_to_end(&mut buffer)?;
    }

    if flags.contains(&'t') {
        list_tar(buffer)?;
    } else if flags.contains(&'x') {
        extract_tar(buffer)?;
    } else {
        eprintln!("No operation specified.");
    }

    Ok(())
}

fn list_tar(buffer: Vec<u8>) -> Result<()> {
    let files = parse_to_file_blocks(&buffer);
    for f in files {
        println!("{}", f.header.file_name);
    }
    Ok(())
}

fn extract_tar(buffer: Vec<u8>) -> Result<()> {
    let file = parse_to_file_blocks(&buffer);
    for f in file {
        let file_name = f.header.file_name.clone();
        let mut file = std::fs::File::create(file_name)?;
        let data = f.data_blocks.concat();
        file.write_all(data.as_slice())?;
    }
    Ok(())
}

