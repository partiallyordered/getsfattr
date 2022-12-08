use xattr;
use std::string::String;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target file name
    #[arg(last=true)]
    file_name: String,
}

fn main() {
    let args = Args::parse();

    let mut xattrs = xattr::list(args.file_name).unwrap().peekable();
    if xattrs.peek().is_none() {
        println!("no xattr set on root");
        return;
    }

    println!("Extended attributes:");
    for attr in xattrs {
        println!(" - {:?}", attr);
    }
}
