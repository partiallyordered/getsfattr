use clap::Parser;
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::str;
use xattr;
use futures::stream::FuturesOrdered;
use futures::StreamExt;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// Get extended file attributes as JSON
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
struct FileAttrs<'a> {
    file_name: &'a PathBuf,
    attrs: HashMap<String, String>,
}

fn build_attrs_for_file(file_name: &PathBuf) -> String {
    let xattr_names = xattr::list::<&OsStr>(file_name.as_ref()).unwrap();
    let attrs = xattr_names
        .map(|xattr_name| (
            xattr_name.clone().into_string().unwrap(),
            str::from_utf8(&xattr::get::<&OsStr, &OsStr>(file_name.as_ref(), xattr_name.as_ref()).unwrap().unwrap()).unwrap().to_owned()
        ))
        .collect();
    serde_json::to_string(&(FileAttrs {
        file_name,
        attrs,
    })).unwrap()
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut stream = FuturesOrdered::new();

    print!("[");

    for f in args.files {
        stream.push_back(
            tokio::spawn(async move {
                build_attrs_for_file(&f)
            })
        )
    }

    while let Some(attrs) = stream.next().await {
        print!("{}", attrs.unwrap());
    }

    println!("]");
}

// Tests:
// - we produce valid json with zero results
// - we produce valid json with one results
// - we produce valid json with two results
// - we print output for files with zero attributes
// - we print output for files with one attribute
// - we print output for files with two attributes
// - what do we do on missing files?
// - what do we do on insufficient permissions?
