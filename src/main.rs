use clap::{Parser, ValueEnum};
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

#[derive(ValueEnum, Clone, Debug, Copy)]
enum Encoding {
    /// Escape attribute values per Rust's std::ascii::escape_default:
    /// https://doc.rust-lang.org/std/ascii/fn.escape_default.html
    Escaped,
    /// Encode attribute values as standard base64, using + and / for 62 and 63 per
    /// https://www.rfc-editor.org/rfc/rfc3548#section-3
    Base64,
    /// Encode attribute values as UTF-8
    Utf8,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// Get extended file attributes as JSON
///
/// Ignores extended file attribute names that cannot be represented as unicode. In practice, this
/// means that all valid attribute names will be supported on Linux, MacOS, and Unix. UTF-8
/// attribute names should still work on Windows.
struct Args {
    /// Extended attribute value encoding. Some values cannot be encoded as strings. The default
    /// behaviour is to produce an error in-line in the output indicating that the value could not
    /// be encoded. This behaviour can be controlled with the --serialize-errors flag.
    #[arg(short, long, default_value="escaped")]
    encoding: Encoding,
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

fn encode_xattr_value(
    xattr_name: String, xattr_value: Vec<u8>, value_encoding: Encoding
) -> Option<Result<(String, String)>> {
    match value_encoding {
        Encoding::Utf8 => match str::from_utf8(&xattr_value) {
            // Returning `None` filters out non-utf-8-representable attribute values.
            // TODO: give the user the option whether to
            // - ignore values that can't be serialized
            // - return an error in the data such as
            //   { "file_name": "test", "error": { "code": 1000, "message": "couldn't serialize" }
            // - return a non-zero exit code
            // - print information on stderr
            Err(_) => None,
            Ok(val) => Some(Ok((xattr_name, val.to_owned()))),
        }
        Encoding::Base64 => Some(Ok((xattr_name, base64::encode(&xattr_value)))),
        Encoding::Escaped => {
            let escaped: Vec<u8> = xattr_value
                .into_iter()
                .flat_map(std::ascii::escape_default).collect();
            match str::from_utf8(&escaped) {
                Err(_) => None,
                Ok(val) => Some(Ok((xattr_name, val.to_owned()))),
            }
        }

    }
}

fn build_attr_json_str_for_file(file_name: PathBuf, value_encoding: Encoding) -> Result<String> {
    let xattr_names = xattr::list::<&OsStr>(file_name.as_ref()).map_err(
        |e| Error::ListExtAttrNames(file_name.clone(), e))?;

    let valid_utf8_xattr_names = xattr_names.filter_map(|xattr_name| xattr_name.into_string().ok());

    let valid_utf8_attrs = valid_utf8_xattr_names.filter_map(|xattr_name| {
        match xattr::get::<&OsStr, &OsStr>(file_name.as_ref(), xattr_name.as_ref()) {
            Err(e) => Some(Err(Error::GetExtAttrValue(xattr_name, file_name.clone(), e))),
            Ok(Some(xattr_value)) => encode_xattr_value(xattr_name, xattr_value, value_encoding),
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
                build_attr_json_str_for_file(first, args.encoding)
            })
        );
        for f in rest.to_owned() {
            stream.push_back(
                tokio::spawn(async move {
                    build_attr_json_str_for_file(f, args.encoding).map(|s| format!(",{s}"))
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
