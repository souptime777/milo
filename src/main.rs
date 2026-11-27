use milo_core::uci::{*};

fn main() {
    let cmd = "gob movetime 10000".parse::<UciCommand>();
    println!("{:?}", cmd);
}
