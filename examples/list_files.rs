use sdw::locker::Locker;
use std::io;

fn main() -> io::Result<()> {
    let locker = Locker::open("/home/wkearn/Documents/data/PANGAEA/HE501")?;

    for entry in locker.dir() {
	let f = entry?;
	println!("{:?}",f.path());
    }
    Ok(())
}
