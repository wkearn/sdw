use sdw::locker::Locker;
use binrw::BinResult;

fn main() -> BinResult<()> {
    let locker = Locker::open("assets/HE501")?;

    let c = locker.iter().filter(|(k,_)| k.0 == "Ping").count();
    println!("{:?}",c);
    Ok(())
}
