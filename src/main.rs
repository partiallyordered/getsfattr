use xattr;
use clap::Parser;
use std::path::PathBuf;
use std::str;
use std::ffi::{OsStr, OsString};
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target files
    #[arg(required = true)]
    files: Vec<PathBuf>,
    // TODO:
    // - ignore missing files
}

#[derive(Debug)]
struct FileAttrs {
    file_name: PathBuf,
    attrs: HashMap<OsString, String>,
}

fn main() {
    let args = Args::parse();

    let results : Vec<FileAttrs> = args.files.into_iter().map(|f| {
        let xattrs = xattr::list::<&OsStr>(f.as_ref()).unwrap();
        FileAttrs {
            file_name: f.clone(),
            attrs: xattrs
                .map(|attr| (
                    attr.clone(),
                    str::from_utf8(&xattr::get::<&OsStr, &OsStr>(f.as_ref(), attr.as_ref()).unwrap().unwrap()).unwrap().to_owned()
                ))
                .collect(),
        }
    }).collect();

    println!("{:?}", results);
}
