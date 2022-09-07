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
use tracing_subscriber;

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
}

#[instrument(skip(sender))]
fn get_file_data(file: PathBuf, sender: Sender<FileData>) -> Result<(), Box<dyn std::error::Error>> {
    let metadata = file.metadata()?;
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
        })?;
        return Ok(());
    }
    else {
        if metadata.is_dir() {
            rayon::spawn(move || {
                info!("Spawned new thread for child directory {}", &file.display());
                let _ = handle_directory(file, sender.clone())
                    .map_err(|e| error!("{}",e));
            });
            return Ok(());
        }
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


#[instrument(skip(sender))]
fn handle_directory(path: PathBuf, sender: Sender<FileData>) -> Result<(), Box<dyn std::error::Error>>
{
    if path.is_symlink() {
        debug!("Skipping symlink {}.", path.display());
        return Ok(());
    }
    let entries = path.read_dir()?;
    let direntries: Vec<(DirEntry,Sender<FileData>)> = entries.filter_map(|s| {
            if s.is_ok() {
                Some((s.unwrap(), sender.clone() ))
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
            let _ = get_file_data(entry.path(), sender)
                .map_err(|e| error!("{}", e));
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
    info!("Opened {} for writing.", &args.outfile.display());
    let start: DateTime<Utc> = Utc::now();
    // Spawn a new thread handling the root directory of the tree
    rayon::spawn(move || { let _ = handle_directory(args.path, sender).map_err(|e| error!("{}",e)); });
    debug!("Spawned root thread.");
    while let Ok(msg) = receiver.recv() {
        debug!("Message received by main thread.");
        wtr.serialize(msg)?;
    }
    wtr.flush()?;
    let end: DateTime<Utc> = Utc::now();
    let diff: chrono::Duration = end - start;
    println!("Finished in {} hours, {} minutes, and {} seconds.", diff.num_hours(), diff.num_minutes(), diff.num_seconds());
    info!("Flushed write buffer.");
    Ok(())
}
