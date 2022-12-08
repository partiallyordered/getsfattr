use xattr;

fn main() {
    let mut xattrs = xattr::list("./trash").unwrap().peekable();
    if xattrs.peek().is_none() {
        println!("no xattr set on root");
        return;
    }

    println!("Extended attributes:");
    for attr in xattrs {
        println!(" - {:?}", attr);
    }
}
