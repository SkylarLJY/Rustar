use super::parser::parse_to_file_blocks;
use std::{
    fs, io::{self, Read, Result, Write}, os::unix::fs::{MetadataExt, PermissionsExt}, time::UNIX_EPOCH
};
use users::{get_group_by_gid, get_user_by_uid};

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

    if flags.contains(&'c') {
        create_tar(&files)?;
        return Ok(());
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

fn create_tar(files: &Vec<&String>) -> Result<()> {
    let mut buffer = Vec::new();
    let tar_file_name = files[0].clone();
    for file_path in files.iter().skip(1) {
        let metadata = fs::metadata(file_path)?;
        // construct header
        let mut file_path_formated = file_path.as_bytes().to_vec();
        file_path_formated.resize(100, 0);
        let file_data = std::fs::read(file_path)?;

        let mut file_mode = format!("{:o}", metadata.permissions().mode())
            .as_bytes()
            .to_vec();
        file_mode.resize(8, 0);

        let mut owner = format!("{:o}", metadata.uid()).as_bytes().to_vec();
        owner.resize(8, 0);

        let mut group = format!("{:o}", metadata.gid())
            .to_string()
            .as_bytes()
            .to_vec();
        group.resize(8, 0);

        let mut file_size = file_data.len().to_string().as_bytes().to_vec();
        file_size.resize(12, 0);

        // let mut last_mod = format!("{:<12}", "0");
        let mut last_mod = metadata
            .modified()?
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs().to_string().as_bytes().to_vec();
        last_mod.resize(12, 0);

        let mut checksum = format!("{:<8}", "        ").into_bytes();

        let file_type = match metadata.file_type() {
            _ if metadata.is_symlink() => "2".as_bytes().to_vec(),
            _ if metadata.is_dir() => "5".as_bytes().to_vec(),
            _ => "0".as_bytes().to_vec(),
        };


        let mut linked_file = if metadata.file_type().is_symlink(){
            // read symlink file 
            let linked_file = std::fs::read_link(&file_path);
            match linked_file{
                Ok(target_path) => {
                    let path_str = target_path.into_os_string().into_string();
                    match path_str{
                        Ok(path) => {
                            path.as_bytes().to_vec()
                        }
                        Err(_) => {
                            Vec::new()
                        }
                    }
                }
                Err(_) => {
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };
        linked_file.resize(100, 0);

        let mut ustar = "ustar".as_bytes().to_vec();
        ustar.resize(6, 0);

        let ustar_version = format!("{:<2}", "00").as_bytes().to_vec();

        let mut owner_name = get_user_by_uid(metadata.uid())
            .map(|user| user.name().to_string_lossy().as_bytes().to_vec())
            .unwrap_or("Unknown".as_bytes().to_vec());
        owner_name.resize(32, 0);

        let mut group_name = get_group_by_gid(metadata.gid())
            .map(|group| group.name().to_string_lossy().as_bytes().to_vec())
            .unwrap_or_else(|| "Unknown".as_bytes().to_vec());
        group_name.resize(32, 0);

        let device_major = vec!['0'; 8].into_iter().map(|c| c as u8).collect();
        let device_minor = vec!['0'; 8].into_iter().map(|c| c as u8).collect();

        let mut prefix = Vec::new();
        prefix.resize(155, 0);

        let mut header_fileds = vec![
            file_path_formated,
            file_mode,
            owner,
            group,
            file_size,
            last_mod,
            checksum,
            file_type,
            linked_file,
            ustar,
            ustar_version,
            owner_name,
            group_name,
            device_major,
            device_minor,
            prefix,
        ];
        let check_sum_val = header_fileds.iter().fold(0, |acc: u64, cur| {
            acc + cur.iter().fold(0, |acc, cur| acc + *cur as u64)
        });
        checksum = format!("{:o}", check_sum_val).as_bytes().to_vec();
        checksum.resize(8, 0);
        header_fileds[6] = checksum;

        let mut file_buffer = Vec::with_capacity(512);
        for field in header_fileds.iter() {
            file_buffer.extend_from_slice(&field);
        }
        file_buffer.resize(512, 0);

        // construct data blocks
        let file_data_blocks = file_data.chunks(512);
        for block in file_data_blocks {
            file_buffer.extend_from_slice(block);
        }
        buffer.extend(file_buffer)
    }
    // write buffer to tar file
    // add two empty blocks to buffer
    buffer.extend(vec![0; 1024]);
    std::fs::write(tar_file_name, buffer)?;
    Ok(())
}
