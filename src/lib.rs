use std::io::Read;

pub fn version() -> &'static str {
    return option_env!("CARGO_PKG_VERSION").unwrap_or("unknown");
}

pub fn build(reader: &mut dyn Read, index: String) -> Result<(), ()> {
    println!("building index '{}'", index);
    unimplemented!();
}

pub fn query(index: String, users: &Vec<String>) -> Result<(), ()> {
    println!("querying users on index '{}':", index);
    for user in users {
        println!("\t{}", user);
    }
    unimplemented!();
}
