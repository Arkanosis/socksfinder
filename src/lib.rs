use byteorder::{
    ReadBytesExt,
    WriteBytesExt,
};

use fst::MapBuilder;

use quick_xml::{
    Reader,
    events::Event,
};

use std::{
    cmp::Reverse,
    collections::{
        BinaryHeap,
        BTreeMap,
        HashSet,
    },
    io::{
        BufRead,
        Read,
        Seek,
        SeekFrom,
        Write,
    },
};

enum Tag {
    Title,
    UserName,
    Other,
}

const SF_IDENTIFIER_LENGTH: usize = 2;
const SF_IDENTIFIER: [u8; SF_IDENTIFIER_LENGTH] = [0x53, 0x46];
const SF_VERSION: u16 = 0;

pub trait Index: BufRead + Seek {}
impl<T: BufRead + Seek> Index for T {}

pub fn version() -> &'static str {
    return option_env!("CARGO_PKG_VERSION").unwrap_or("unknown");
}

pub fn build(reader: &mut dyn BufRead, writer: &mut dyn Write) -> Result<(), ()> {
    writer.write_all(&SF_IDENTIFIER).unwrap();
    writer.write_u16::<byteorder::LittleEndian>(SF_VERSION).unwrap();
    let mut current_offset = 4u32;
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
                                writer.write_u8(0xA).unwrap();
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
            Err(error) => {
                eprintln!("socksfinder: XML parsing error at position {}: {:?}", xml_reader.buffer_position(), error);
                break;
            },
            Ok(Event::Eof) => break,
            _ => (),
        }
        buffer.clear();
    }
    current_offset += previous_page_length as u32;
    for page_offsets in user_page_offsets.values_mut() {
        let edit_count = page_offsets.len() as u32;
        let page_offsets_offset = current_offset;
        for page_offset in page_offsets.iter() {
            writer.write_u32::<byteorder::LittleEndian>(*page_offset).unwrap();
        }
        current_offset += (page_offsets.len() as u32) * 4;
        page_offsets.clear();
        page_offsets.push(page_offsets_offset);
        page_offsets.push(edit_count);
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

struct InvertedList<'a> {
    user: &'a String,
    position: usize,
    page_offsets: Vec<u32>,
}

pub fn query(index: &mut dyn Index, users: &Vec<String>, threshold: usize) -> Result<(), ()> {
    let mut identifier_bytes = [0u8; SF_IDENTIFIER_LENGTH];
    match index.read(&mut identifier_bytes) {
        Ok(length) => {
            if length != SF_IDENTIFIER_LENGTH ||
               identifier_bytes != SF_IDENTIFIER {
                   eprintln!("socksfinder: not a socksfinder index");
                   return Err(())
            }
        },
        Err(_) => {
            eprintln!("socksfinder: not a socksfinder index");
            return Err(())
        }
    }
    match index.read_u16::<byteorder::LittleEndian>() {
        Ok(index_version) => {
            if index_version != SF_VERSION {
                eprintln!("socksfinder: can't read index in format version {}, only format version {} is supported by socksfinder v{}", index_version, SF_VERSION, version());
                return Err(())
            }
        },
        Err(_) => {
            eprintln!("socksfinder: unable to read index format version number");
            return Err(())
        }
    }
    let fst_end_offset = index.seek(SeekFrom::End(-4)).unwrap();
    let fst_start_offset = index.read_u32::<byteorder::LittleEndian>().unwrap();
    index.seek(SeekFrom::Start(fst_start_offset as u64)).unwrap();
    let mut fst_reader = index.take(fst_end_offset - fst_start_offset as u64);
    let mut fst_bytes = vec![];
    fst_reader.read_to_end(&mut fst_bytes).unwrap();
    let fst = fst::Map::from_bytes(fst_bytes).unwrap();
    let mut lists = vec![];
    let mut min_page_offsets = HashSet::with_capacity(users.len());
    for user in users {
        match fst.get(&user) {
            None => {
                eprintln!("User '{}' does not exist or has no contribution ", user);
                return Err(());
            },
            Some(value) => {
                let edit_count = value & 0xFF_FF_FF_FF;
                let page_offsets_offset = value >> 32;
                index.seek(SeekFrom::Start(page_offsets_offset)).unwrap();
                let mut page_offsets = Vec::<u32>::with_capacity(edit_count as usize);
                for _ in 0..edit_count {
                    page_offsets.push(index.read_u32::<byteorder::LittleEndian>().unwrap());
                }
                lists.push(InvertedList {
                    user: user,
                    position: 0,
                    page_offsets: page_offsets
                });
                min_page_offsets.insert(lists.last().unwrap().page_offsets[0]);
            }
        }
    }
    let mut heap = BinaryHeap::with_capacity(min_page_offsets.len());
    for min_page_offset in min_page_offsets {
        heap.push(Reverse(min_page_offset));
    }
    let mut page_name = String::new();
    let mut editors = Vec::with_capacity(users.len());
    while !heap.is_empty() {
        let Reverse(current_page_offset) = heap.pop().unwrap();
        index.seek(SeekFrom::Start(current_page_offset as u64)).unwrap();
        index.read_line(&mut page_name).unwrap();
        page_name.pop();
        let mut editors_count = 0;
        for list in &mut lists {
            if list.page_offsets[list.position] == current_page_offset {
                editors_count += 1;
                editors.push(list.user);
                if list.position < list.page_offsets.len() - 1 {
                    list.position += 1;
                    heap.push(Reverse(list.page_offsets[list.position]));
                }
            }
        }
        if editors_count >= threshold {
            let mut editor_names = String::with_capacity(editors.len() * 20);
            for editor in &editors {
                editor_names.push_str(editor);
                editor_names.push_str(", ");
            }
            editor_names.truncate(editor_names.len() - 2);
            println!("{}: {} ({})", page_name, editors_count, editor_names);
        }
        page_name.clear();
        // TODO update co-occurence matrix
        editors.clear();
    }
    // TODO write co-occurence matrix
    Ok(())
}
