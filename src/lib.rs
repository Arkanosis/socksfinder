use byteorder::WriteBytesExt;

use fst::MapBuilder;

use quick_xml::{
    Reader,
    events::Event,
};

use std::collections::BTreeMap;

use std::io::{
    BufRead,
    Write,
};

enum Tag {
    Title,
    UserName,
    Other,
}

const SF_IDENTIFIER: [u8; 2] = [0x53, 0x46];
const SF_VERSION: u16 = 0;

pub fn version() -> &'static str {
    return option_env!("CARGO_PKG_VERSION").unwrap_or("unknown");
}

pub fn build(reader: &mut dyn BufRead, writer: &mut dyn Write) -> Result<(), ()> {
    writer.write_all(&SF_IDENTIFIER).unwrap();
    writer.write_u16::<byteorder::LittleEndian>(SF_VERSION).unwrap();

    let mut current_offset = 0u32;
    let mut user_page_offsets = BTreeMap::new();
    let mut xml_reader = Reader::from_reader(reader);
    let mut buffer = Vec::new();
    let mut current_tag = Tag::Other;
    let mut previous_page_length = 0usize;
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
            Ok(Event::Text(ref event)) => {
                match current_tag {
                    Tag::Title => {
                        match event.unescaped() {
                            Ok(ref buffer) => {
                                current_offset += previous_page_length as u32;
                                writer.write_all(buffer).unwrap();
                                writer.write_u8(0).unwrap();
                                previous_page_length = buffer.len() + 1;
                            }
                            Err(_) => (), // ignore encoding error in the dump
                        }
                    }
                    Tag::UserName => {
                        match event.unescaped() {
                            Ok(ref buffer) => {
                                let page_offsets = user_page_offsets.entry(buffer.to_vec()).or_insert(Vec::new());
                                match page_offsets.last() {
                                    None => page_offsets.push(current_offset),
                                    Some(last_offset) => {
                                        if current_offset != *last_offset {
                                            page_offsets.push(current_offset);
                                        }
                                    }
                                }
                            },
                            Err(_) => (), // ignore encoding error in the dump
                        }
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
    current_offset += previous_page_length as u32;
    for page_offsets in user_page_offsets.values_mut() {
        for page_offset in page_offsets.iter() {
            writer.write_u32::<byteorder::LittleEndian>(*page_offset).unwrap();
        }
        current_offset += (page_offsets.len() as u32) * 4;
        page_offsets.clear();
        page_offsets.push(current_offset);
        page_offsets.push(page_offsets.len() as u32);
    }
    let fst_offset = current_offset;
    let mut fst_builder = MapBuilder::new(writer).unwrap();
    for (user, page_offsets) in user_page_offsets {
        fst_builder.insert(user, (page_offsets[0] as u64) << 32 | (page_offsets[1] as u64)).unwrap();
    }
    let writer = fst_builder.into_inner().unwrap();
    writer.write_u32::<byteorder::LittleEndian>(fst_offset).unwrap();
    Ok(())
}

pub fn query(index: String, users: &Vec<String>) -> Result<(), ()> {
    // TODO check magic number
    // TODO check version number
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
