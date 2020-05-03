use byteorder::WriteBytesExt;

use quick_xml::{
    Reader,
    events::Event,
};

use std::io::{
    BufRead,
    Write,
};

enum Tag {
    Title,
    UserName,
    Other,
}

pub fn version() -> &'static str {
    return option_env!("CARGO_PKG_VERSION").unwrap_or("unknown");
}

pub fn build(reader: &mut dyn BufRead, writer: &mut dyn Write) -> Result<(), ()> {
    let mut xml_reader = Reader::from_reader(reader);
    let mut buffer = Vec::new();
    let mut current_tag = Tag::Other;
    loop {
        match xml_reader.read_event(&mut buffer) {
            Ok(Event::Start(ref event)) => {
                match event.name() {
                    b"title" => current_tag = Tag::Title,
                    b"username" => current_tag = Tag::UserName,
                    _ => current_tag = Tag::Other,
                }
            },
            Ok(Event::End(_)) => current_tag = Tag::Other,
            Ok(Event::Text(event)) => {
                match current_tag {
                    Tag::Title => {
                        print!("page: '{}'\n", event.unescape_and_decode(&xml_reader).unwrap());
                        match event.unescaped() {
                            Ok(ref buffer) => {
                                // TODO keep previous offset for index
                                writer.write_all(buffer).unwrap();
                                writer.write_u8(0).unwrap();
                            }
                            Err(_) => (), // ignore encoding error in the dump
                        }
                    }
                    Tag::UserName => {
                        print!("\tuser: '{}'\n", event.unescape_and_decode(&xml_reader).unwrap());
                        // TODO add user to map if not present, with an empty u32 list
                        // TODO add previous page name offset to user list
                    },
                    Tag::Other => (),
                }
            },
            Err(error) => panic!("XML parsing error at position {}: {:?}", xml_reader.buffer_position(), error),
            Ok(Event::Eof) => break,
            _ => (),
        }
        buffer.clear();
    }
    // TODO for each user (alphabetically)
    // TODO   keep previous offset for index
    // TODO   write page offsets list length
    // TODO   write page offsets list
    // TODO keep previous offset for header
    // TODO write username -> user offset mapping as FST
    // TODO write FST offset as u32
    Ok(())
}

pub fn query(index: String, users: &Vec<String>) -> Result<(), ()> {
    println!("querying users on index '{}':", index);
    // TODO read last u32 -> offset of the FST
    // TODO mmap FST
    for user in users {
        println!("\t{}", user);
        // TODO lookup user in FST -> page offsets list offset for that user
        // TODO add (username, offset) to list
    }
    unimplemented!();
    // TODO heap merge of all lists
    // TODO while ! heap.is_empty()
    // TODO   while heap.peek() is same
    // TODO     keep users
    // TODO     update co-occurrence matrix
    // TODO   write CSV line with page name, number of users, user names
    // TODO write CSV co-occurence matrix
}
