use clap::Parser;
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::str;
use xattr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target files
    #[arg(required = true)]
    files: Vec<PathBuf>,
    // TODO:
    // - ignore missing files
    // - pretty-print output
    // - attr filters? (probably not- just do one thing well)
    // - traversal options? (probably not- just do one thing well)
}

#[derive(Debug, Serialize)]
struct FileAttrs {
    file_name: PathBuf,
    attrs: HashMap<String, String>,
}

fn main() {
    let args = Args::parse();

    let results : Vec<FileAttrs> = args.files.into_iter().map(|f| {
        let xattrs = xattr::list::<&OsStr>(f.as_ref()).unwrap();
        FileAttrs {
            file_name: f.clone(),
            attrs: xattrs
                .map(|attr| (
                    attr.clone().into_string().unwrap(),
                    str::from_utf8(&xattr::get::<&OsStr, &OsStr>(f.as_ref(), attr.as_ref()).unwrap().unwrap()).unwrap().to_owned()
                ))
                .collect(),
        }
    }).collect();

    println!("{}", serde_json::to_string(&results).unwrap());
}
