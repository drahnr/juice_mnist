#![feature(backtrace)]

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use curl::easy::Easy;
use docopt::Docopt;
use flate2::read::GzDecoder;
use serde::Deserialize;
use url::Url;

use crate::error::Error;

mod error;

const DATA_LINKS: [&str; 4] = [
    "http://yann.lecun.com/exdb/mnist/train-images-idx3-ubyte.gz",
    "http://yann.lecun.com/exdb/mnist/train-labels-idx1-ubyte.gz",
    "http://yann.lecun.com/exdb/mnist/t10k-images-idx3-ubyte.gz",
    "http://yann.lecun.com/exdb/mnist/t10k-labels-idx1-ubyte.gz",
];

const USAGE: &'static str = "
Juice MNIST Example

Usage:
juice_mnist download <directory>
juice_mnist train
juice_mnist test <network config>
juice_mnist (-h | --help)

Commands:
  download      Downloads the datasets
  train         Trains the network and saves it based on the data in the \"data\" directory
  test <config> Tests the saved data based on the passed in config generated by --train.

Options:
  -h --help     Show this screen
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_download: bool,
    cmd_train: bool,
    cmd_test: bool,
    arg_directory: Option<String>,
    arg_config: Option<String>,
}

fn download_datasets(dir: Option<String>) -> Result<(), Error> {
    let directory = dir.ok_or("Please specify a directory when downloading datasets")?;

    let mut easy = Easy::new();

    for s in DATA_LINKS.iter() {
        let s = *s;
        let url = Url::parse(s)?;
        let filename: &str = url
            .path_segments()
            .ok_or("Could not get path segments")?
            .last()
            .ok_or("Could not get last path segment")?;

        let path = Path::new::<str>(directory.as_ref()).join(filename);
        let path = path.as_path();
        println!("Downloading {:?}", path.file_name().unwrap());
        {
            let mut file = File::create(path).expect("Failed to create a file to write to");

            easy.url(s)?;
            easy.write_function(move |data| {
                file.write_all(&data).unwrap();
                Ok(data.len())
            })?;
            easy.perform().unwrap();
        }

        {
            let mut gzippedBytes: Vec<u8> = vec![];
            let file = File::open(path)?;
            let mut decoder = GzDecoder::new(file);
            println!("Decoding file {:?}", path.file_name().unwrap());
            decoder.read_to_end(&mut gzippedBytes);

            // Write the decoded file
            let path = Path::new::<str>(directory.as_ref()).join(filename.replace(".gz", ""));
            let mut unzipped_file = File::create(path)?;
            unzipped_file.write_all(gzippedBytes.as_slice())?;
        }
    }
    Ok(())
}

fn main() -> Result<(), Error> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    println!("{:?}", args);

    if args.cmd_download {
        download_datasets(args.arg_directory)?;
    }

    Ok(())
}