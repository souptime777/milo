use milo_core::uci::UciCommand;

fn main() {
    let cmd= "setoption strength 1000".parse::<UciCommand>();
    println!("{:?}", cmd);
}
