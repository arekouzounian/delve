use delve_macros::entity;

#[entity]
pub struct Item {
    item: u64,
}

fn main() {
    let i = Item::default();

    println!("{:?}", i);
}
