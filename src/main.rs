use clap::Parser;
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::str;
use tokio::sync::mpsc;
use xattr;
use rand::{thread_rng, Rng};

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

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    print!("[");

    if args.files.len() > 0 {
        // Extract a random item from the file list so we can print that after all the others, with
        // no trailing comma. We remove a random item because the files will be returned
        // out-of-order. We return files out-of-order because we process them concurrently and do
        // not perform any sorting. This has the following advantages:
        // - concurrency means the program should finish more quickly for a large number of files
        // - not sorting means we can stream all output as it arrives, reducing memory usage
        //   (though this *could* be mitigated by sorting on arrival and printing all items that
        //   are printable- e.g. if items 1-3, 5 and 11 have arrived, we can print 1-3, etc.- this
        //   will not be done at the time of writing..)
        // - not sorting means we don't need to implement/maintain sorting code
        let mut rng = thread_rng();
        let mut rest = args.files.to_owned();
        let last_file = rest.swap_remove(rng.gen_range(0..args.files.len()));

        for f in rest {
            let tx = tx.clone();
            tokio::spawn(async move {
                tx.send(build_attrs_for_file(&f));
            });
        }

        drop(tx); // Drop tx here so the receive channel closes when all the attributes have been received.
        while let Some(s) = rx.recv().await {
            // TODO: when does print flush? Should we ever flush manually? Expose this to the user?
            print!("{},", s);
        }

        print!("{}", build_attrs_for_file(&last_file));
    }

    println!("]");
}
