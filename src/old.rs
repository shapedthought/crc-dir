use std::{fs::read, path::PathBuf, sync::{Arc, Mutex}};
use crc32fast::Hasher;
use anyhow::Result;
use jwalk::WalkDir;
use indicatif::ProgressBar;
use rayon::prelude::*;

use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[clap(short, long, value_parser)]
    path: PathBuf,

    #[clap(short, long, default_value_t = usize::MAX, value_parser)]
    depth: usize
}

fn main() -> Result<()> {
    let cli = Cli::parse();


    let hasher = Arc::new(Mutex::new(Hasher::new()));

    let files: Vec<_> = WalkDir::new(cli.path)
        .max_depth(cli.depth)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|d| d.file_type().is_file())
        .collect();

    let bar = ProgressBar::new(files.len() as u64);

    files.par_iter().for_each(|e| {

        let file_buf = read(e.path()).unwrap();

        hasher.lock().unwrap().update(&file_buf);

        bar.inc(1);
    });

    bar.finish();

    // let mutex = hasher.lock().unwrap();

    let checksum = Arc::try_unwrap(hasher).unwrap().into_inner().unwrap().finalize();

    println!("{:x}", checksum);

    Ok(())
}
