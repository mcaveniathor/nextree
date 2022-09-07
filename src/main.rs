extern crate clap;
use clap::Parser;
extern crate csv;
use csv::Writer;
extern crate rayon;
use rayon::prelude::*;
extern crate serde;
use serde::{Serialize,Deserialize};
extern crate chrono;
use chrono::prelude::*;
#[macro_use] extern crate tracing;

use std::{
    path::PathBuf,
    fs::DirEntry,
    sync::mpsc::{Sender, Receiver, channel},
};

#[derive(Debug, Deserialize, Serialize)]
struct FileData {
    path: String,
    created: Option<DateTime<Utc>>,
    modified: Option<DateTime<Utc>>,
    // size in bytes
    size: u64,
}

fn get_file_data(file: PathBuf, sender: Sender<FileData>) -> Result<(), Box<dyn std::error::Error>> {
    let metadata = file.metadata().map_err(|e| format!("{}:{}", e, file.display()))?;
    if metadata.is_symlink() {
        return Ok(());
    }
    if metadata.is_file() {
        let path = String::from(file.to_string_lossy());
        let (created,modified) = match (metadata.created(), metadata.modified()) {
            (Ok(c), Ok(m)) => { (Some(c.into()), Some(m.into())) },
            (Ok(c), Err(_)) => { (Some(c.into()), None) },
            (Err(_), Ok(m)) => { (None, Some(m.into())) },
            _ => { (None,None) }
        };
        sender.send(FileData {
            path,
            created,
            modified,
            size: metadata.len(),
        })?;
        return Ok(());
    }
    else if metadata.is_dir() {
            rayon::spawn(move || {
                let span = debug_span!("handle_directory", "{}", &file.display());
                let _enter = span.enter();
                let _ = handle_directory(file, sender)
                    .map_err(|e| debug!("{}",e));
            });
            return Ok(());
    }
    Ok(())
}

#[derive(Parser,Debug)]
struct Args {
    #[clap(short,long)]
    /// Root path whose children we want to log
    path: PathBuf,
    #[clap(short,long, default_value_os_t=PathBuf::from("out.csv"))]
    /// CSV file to output to
    outfile: PathBuf,
}


fn handle_directory(path: PathBuf, sender: Sender<FileData>) -> Result<(), Box<dyn std::error::Error>>
{
    if path.is_symlink() {
        debug!("Skipping symlink {}.", path.display());
        return Ok(());
    }
    let entries = path.read_dir()?;
    let direntries: Vec<(DirEntry,Sender<FileData>)> = entries.filter_map(|s| {
            if let Ok(s) = s {
                Some((s, sender.clone() ))
            }
            else {
                None
            }
    })
    .collect();
    direntries
        // Perform this operation in parallel according to available resources
        .into_par_iter()
        .for_each(|(entry, sender)| {
            let span = debug_span!("entry", "{}", entry.path().display());
            let _ = span.enter();
            let _ = get_file_data(entry.path(), sender)
                .map_err(|e| debug!("{}", e));
        });
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    debug!("Logger initialized.");
    let args = Args::parse();
    // receiver to be used by thread collecting FileData structs for serialization
    let (sender, receiver): (Sender<FileData>, Receiver<FileData>) = channel();
    let mut wtr = Writer::from_path(&args.outfile)?;
    debug!("Opened {} for writing.", &args.outfile.display());
    let start: DateTime<Utc> = Utc::now();
    // Spawn a new thread handling the root directory of the tree
    rayon::spawn(move || {
        let span = debug_span!("root thread");
        let _enter = span.enter();
        let _ = handle_directory(args.path.clone(), sender)
            .map_err(|e| debug!("{}",e));
    });
    debug!("Spawned root thread.");
    let mut count = 0;
    let mut size = 0;
    while let Ok(msg) = receiver.recv() {
        size += msg.size;
        count += 1;
        let _err = wtr.serialize(msg).map_err(|e| error!("{}",e));
    }
    wtr.flush()?;
    let end: DateTime<Utc> = Utc::now();
    let diff: chrono::Duration = end - start;
    let (h,m,s,ms) = {
        let h = diff.num_hours();
        diff -= Duration::hours(h);
        let m = diff::num_minutes();
        diff -= Duration::minutes(m);
        let ms = diff.num_milliseconds();
        (h,m,ms)
    };
        
        
    info!("Reported on {} files totalling {} bytes in {} hours, {} minutes, {} seconds, and {} milliseconds.",  count, size,
        h, m, ms);
    debug!("Flushed write buffer.");
    Ok(())
}
