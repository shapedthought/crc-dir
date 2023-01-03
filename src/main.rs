use anyhow::Result;
use indicatif::ProgressBar;
use jwalk::WalkDir;
use rayon::prelude::*;
use std::{
    ffi::OsStr,
    fs::read,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use std::time::Instant;
use csv::Writer;

use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[clap(short, long, value_parser)]
    path: PathBuf,

    #[clap(short, long, default_value_t = usize::MAX, value_parser)]
    depth: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct FileInfo<'a> {
    name: &'a OsStr,
    crc: String,
}

impl<'t> FileInfo<'t> {
    fn new(name: &'t OsStr, file_buf: &Vec<u8>) -> Self {
        let checksum = crc32fast::hash(&file_buf);
        let crc = format!("{:x}", checksum);
        FileInfo {
            name,
            crc,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let start = Instant::now();

    let files: Vec<_> = WalkDir::new(cli.path)
        .max_depth(cli.depth)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|d| d.file_type().is_file())
        .collect();

    let bar = ProgressBar::new(files.len() as u64);

    let file_infos = Arc::new(Mutex::new(Vec::new()));

    files.par_iter().for_each(|e| {
        let file_buf = read(e.path()).unwrap();

        let file_name = e.file_name();

        let file_info = FileInfo::new(file_name, &file_buf);

        file_infos.lock().unwrap().push(file_info);
        bar.inc(1);
    });

    bar.finish();

    let mut results = file_infos.lock().unwrap();

    results.sort();

    let mut wtr = Writer::from_path("crc-dir.csv")?;
    wtr.write_record(&["Name", "CRC"])?;

    for item in results.iter() {
        wtr.write_record(&[item.name.to_str().unwrap(), &item.crc])?;
    }

    wtr.flush()?;

    let duration = start.elapsed();
    println!("Time elapsed: {:?}, files: {}", duration, results.len());

    Ok(())
}
