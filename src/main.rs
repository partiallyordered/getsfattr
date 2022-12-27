use clap::Parser;
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::ExitCode;
use std::str;
use xattr;
use futures::stream::FuturesOrdered;
use futures::StreamExt;
use thiserror::Error;

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
    // - take files as json input in some form? (similar to our output form..?)
}

type FileAttrsMap = HashMap<String, String>;

#[derive(Debug, Serialize)]
struct FileAttrs<'a> {
    file_name: &'a PathBuf,
    attrs: FileAttrsMap,
}

#[derive(Error, Debug)]
enum Error {
    #[error("Failed to get extended attributes for file {0}: {1}")]
    ListExtAttrNames(PathBuf, std::io::Error),
    #[error("Failed to get extended attribute {0} value for file {1}: {2}")]
    GetExtAttrValue(String, PathBuf, std::io::Error),
    #[error("Extended attribute {0} does not have a value for file {1}")]
    NoExtAttrValue(String, PathBuf),
    #[error("Failed to serialize attributes to JSON for file {0}. Error: {1}")]
    FailedToSerializeAttrs(PathBuf, serde_json::Error),
}

type Result<T> = std::result::Result<T, Error>;

fn build_attr_json_str_for_file(file_name: PathBuf) -> Result<String> {
    let xattr_names = xattr::list::<&OsStr>(file_name.as_ref()).map_err(
        |e| Error::ListExtAttrNames(file_name.clone(), e))?;

    let valid_utf8_xattr_names = xattr_names.filter_map(|xattr_name| xattr_name.into_string().ok());

    let valid_utf8_attrs = valid_utf8_xattr_names.filter_map(|xattr_name| {
        match xattr::get::<&OsStr, &OsStr>(file_name.as_ref(), xattr_name.as_ref()) {
            Err(e) => Some(Err(Error::GetExtAttrValue(xattr_name, file_name.clone(), e))),
            Ok(Some(xattr_value)) => match str::from_utf8(&xattr_value) {
                Err(_) => None, // filter out non-utf-8-representable attribute values
                Ok(val) => Some(Ok((xattr_name, val.to_owned()))),
            },
            Ok(None) => Some(Err(Error::NoExtAttrValue(xattr_name, file_name.clone())))
        }
    }).collect::<Result<FileAttrsMap>>()?;

    serde_json::to_string(&(FileAttrs {
        file_name: &file_name,
        attrs: valid_utf8_attrs,
    })).map_err(|e| Error::FailedToSerializeAttrs(file_name, e))
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    let mut stream = FuturesOrdered::new();

    print!("[");

    if let Some((first, rest)) = args.files.split_first() {
        let first = first.to_owned();
        stream.push_back(
            tokio::spawn(async move {
                build_attr_json_str_for_file(first)
            })
        );
        for f in rest.to_owned() {
            stream.push_back(
                tokio::spawn(async move {
                    build_attr_json_str_for_file(f).map(|s| format!(",{s}"))
                })
            )
        }
    }

    while let Some(attrs) = stream.next().await {
        match attrs {
            Ok(Ok(attrs_inner)) => print!("{}", attrs_inner),
            Ok(Err(e)) => {
                eprintln!("Error!: {}", e);
                return ExitCode::FAILURE;
            }
            Err(e) => {
                eprintln!("Error!: {}", e);
                return ExitCode::FAILURE;
            }
        }
    }

    println!("]");

    ExitCode::SUCCESS
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
