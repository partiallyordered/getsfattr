use xattr;
use clap::Parser;
use std::path::PathBuf;
use std::str;
use std::ffi::OsStr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target files
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();

    for f in args.files {
        let mut xattrs = xattr::list::<&OsStr>(f.as_ref()).unwrap().peekable();
        if xattrs.peek().is_none() {
            println!("no xattr set on {:?}", f);
            return;
        }

        println!("Extended attributes:");
        for attr in xattrs {
            let val = xattr::get::<&OsStr, &OsStr>(f.as_ref(), attr.as_ref());
            println!(" - {:?} := {:?}", attr, str::from_utf8(&val.unwrap().unwrap()).unwrap());
        }
    }
}
