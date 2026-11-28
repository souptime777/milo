
use milo_core::uci::{UciOption, UciOptionType};
fn main() {
    let option = UciOption::new(
        "Nullmove",
        UciOptionType::Check {default: true},
    );
    println!("{}", option);
}
