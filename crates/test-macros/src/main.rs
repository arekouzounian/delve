use delve_macros::entity;

#[derive(Debug)]
#[entity]
pub struct Item {
    item: u64,
}

fn main() {
    let i = Item::default();

    println!("{:?}", i);
}
