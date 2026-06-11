use delve_macros::entity;

#[entity({
    asdf,
    asdf2,
    asdf3
})]
pub struct Item {
    item: u64,
}

fn main() {
    println!("Hello, world!");
}
