use actix_files::NamedFile;

use actix_web::{
    get,
    http::header::ContentType,
    web::{
        Data,
        Json,
        Query,
    },
    App,
    HttpResponse,
    HttpServer,
    Responder,
    Result as WebResult,
};

use askama_actix::Template;

use byteorder::{
    ReadBytesExt,
    WriteBytesExt,
};

use number_prefix::NumberPrefix;

use fst::{
    MapBuilder,
    Streamer,
};

use prettytable::{
    Cell,
    Row,
    Table,
};

use quick_xml::{
    Reader,
    events::Event,
};

use mime::TEXT_PLAIN_UTF_8;

use serde_derive::{
    Deserialize,
    Serialize,
};

use std::{
    cmp::Reverse,
    collections::{
        BinaryHeap,
        BTreeMap,
        HashMap,
        HashSet,
    },
    fs::File,
    io::{
        BufRead,
        Cursor,
        Read,
        Seek,
        SeekFrom,
        Write,
    },
    path::Path,
    sync::Mutex,
    time::Instant,
};

enum Tag {
    Title,
    UserName,
    Other,
}

#[allow(non_camel_case_types)]
#[derive(Clone)]
#[derive(Copy)]
#[derive(Deserialize)]
#[derive(PartialEq)]
pub enum Order {
    alphabetical,
    count_decreasing,
    count_increasing,
    none,
}

const SF_IDENTIFIER_LENGTH: usize = 2;
const SF_IDENTIFIER: [u8; SF_IDENTIFIER_LENGTH] = [0x53, 0x46];
const SF_VERSION: u16 = 0;

pub trait Index: BufRead + Seek {}
impl<T: BufRead + Seek> Index for T {}

pub fn version() -> &'static str {
    if env!("CARGO_PKG_VERSION").ends_with("-dev") {
        concat!(env!("CARGO_PKG_VERSION"), "+", env!("VERGEN_GIT_SHA_SHORT"))
    } else {
        env!("CARGO_PKG_VERSION")
    }
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
                    b"ip" => current_tag = Tag::UserName,
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

struct Page {
    page_name: String,
    editor_count: usize,
    editor_names: String,
}

fn read_index_header(index: &mut dyn Index) -> Result<(u64, u64), ()> {
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
    Ok((fst_start_offset as u64, fst_end_offset))
}

pub fn query(index: &mut dyn Index, writer: &mut dyn Write, users: &Vec<String>, threshold: usize, order: Order, show_cooccurrences: bool, try_format: bool) -> Result<(), ()> {
    let users: &Vec<String> = {
        let mut unique_users = HashSet::with_capacity(users.len());
        for user in users {
            unique_users.insert(user.trim().to_string().clone());
        }
        &unique_users.into_iter().collect()
    };
    let threshold = if show_cooccurrences {
        0
    } else if threshold == 0 {
        users.len()
    } else {
        threshold
    };
    let (fst_start_offset, fst_end_offset) = read_index_header(index)?;
    index.seek(SeekFrom::Start(fst_start_offset)).unwrap();
    let mut fst_reader = index.take(fst_end_offset - fst_start_offset);
    let mut fst_bytes = vec![];
    fst_reader.read_to_end(&mut fst_bytes).unwrap();
    let fst = fst::Map::new(fst_bytes).unwrap();
    let mut lists = vec![];
    let mut min_page_offsets = HashSet::with_capacity(users.len());
    let mut error = false;
    for user in users {
        match fst.get(&user) {
            None => {
                error = true;
                match write!(writer, "Error: User '{}' does not exist or has no edits\n", user) {
                    Ok(()) => (),
                    Err(_) => (), // ignore output error, but give up
                }
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
                    user,
                    position: 0,
                    page_offsets
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
    let mut cooccurrences = if show_cooccurrences {
        // TODO only half of the matrix is actually necessary
        HashMap::with_capacity(users.len() * users.len())
    } else {
        HashMap::new()
    };
    let mut list_count = lists.len();
    let mut pages = Vec::new();
    while !heap.is_empty() &&
          list_count >= threshold {
        let Reverse(current_page_offset) = heap.pop().unwrap();
        let mut editor_count = 0;
        for list in &mut lists {
            if list.page_offsets[list.position] == current_page_offset {
                editor_count += 1;
                editors.push(list.user);
                if list.position < list.page_offsets.len() - 1 {
                    list.position += 1;
                    heap.push(Reverse(list.page_offsets[list.position]));
                } else {
                    list_count -= 1;
                }
            }
        }
        if show_cooccurrences && editors.len() > 1 {
            for first_editor in &editors {
                for second_editor in &editors {
                    cooccurrences.entry((first_editor.clone(), second_editor.clone())).and_modify(|value| { *value += 1 }).or_insert(1);
                }
            }
        } else if editor_count >= threshold {
            index.seek(SeekFrom::Start(current_page_offset as u64)).unwrap();
            index.read_line(&mut page_name).unwrap();
            page_name.pop();
            let mut editor_names = String::with_capacity(editors.len() * 20);
            for editor in &editors {
                editor_names.push_str(editor);
                editor_names.push_str(", ");
            }
            editor_names.truncate(editor_names.len() - 2);
            match order {
                Order::none => {
                    match write!(writer, "{}: {} ({})\n", page_name, editor_count, editor_names) {
                        Ok(()) => (),
                        Err(_) => (), // ignore output error, but give up
                    }
                },
                _ => pages.push(Page {
                    page_name: page_name.clone(),
                    editor_count,
                    editor_names
                }),
            }
            page_name.clear();
        }
        editors.clear();
    }
    if show_cooccurrences {
        let mut sorted_users = users.clone();
        if order != Order::none {
            sorted_users.sort_unstable_by(|first_user, second_user| {
                if order == Order::alphabetical {
                    first_user.cmp(&second_user)
                } else {
                    let total = |user: &String| {
                        let mut sum = 0;
                        for other_user in users {
                            if other_user != user {
                                sum += cooccurrences.get(&(&user.clone(), &other_user.clone())).unwrap_or(&0);
                            }
                        }
                        sum
                    };
                    if order == Order::count_decreasing {
                        total(second_user).cmp(&total(first_user))
                    } else {
                        total(first_user).cmp(&total(second_user))
                    }
                }
            });
        }
        let mut table = Table::new();
        let mut row = vec![Cell::new("")];
        for user in &sorted_users {
            row.push(Cell::new(&user).style_spec("b"));
        }
        table.add_row(Row::new(row));
        for row_user in &sorted_users {
            let mut row = vec![Cell::new(&row_user).style_spec("b")];
            for cell_user in &sorted_users {
                if row_user == cell_user {
                    row.push(Cell::new(""));
                } else {
                    row.push(Cell::new(&(cooccurrences.get(&(&row_user.clone(), &cell_user.clone())).unwrap_or(&0)).to_string()));
                }
            }
            table.add_row(Row::new(row));
        }
        if try_format {
            table.printstd();
        } else {
            match table.print(writer) {
                Ok(_) => (),
                Err(_) => (), // ignore output error
            }
        }
    } else {
        match order {
            Order::none => (),
            _ => {
                pages.sort_unstable_by(|first_page, second_page| {
                    match order {
                        Order::alphabetical => first_page.page_name.cmp(&second_page.page_name),
                        Order::count_decreasing => second_page.editor_count.cmp(&first_page.editor_count),
                        Order::count_increasing => first_page.editor_count.cmp(&second_page.editor_count),
                        _ => unreachable!()
                    }
                });
                for page in pages {
                    match write!(writer, "{}: {} ({})\n", page.page_name, page.editor_count, page.editor_names) {
                        Ok(()) => (),
                        Err(_) => break, // ignore output error, but give up
                    }
                }
            }
        }
    }
    if error {
        Err(())
    } else {
        Ok(())
    }
}

struct RamIndex {
    name: String,
    data: Vec<u8>,
}

struct AppState {
    index_path: String,
    ram_index: Mutex<RamIndex>,
}

#[derive(Template)]
#[template(path = "index.htm")]
struct IndexTemplate<'a> {
    version: &'a str,
    index: String,
}

#[get("/")]
async fn serve_index(data: Data<AppState>) -> impl Responder {
    let ram_index = data.ram_index.lock().unwrap();
    IndexTemplate {
        version: version(),
        index: ram_index.name.clone(),
    }
}

#[allow(non_snake_case)]
#[derive(Serialize)]
struct BadgeResponse {
    label: String,
    message: String,
    schemaVersion: u32,
}

#[get("/badge")]
async fn serve_badge(_data: Data<AppState>) -> WebResult<Json<BadgeResponse>> {
    Ok(Json(BadgeResponse {
        label: "socksfinder".to_string(),
        message: version().to_string(),
        schemaVersion: 1,
    }))
}

#[get("/comparison")]
async fn serve_comparison(_data: Data<AppState>) -> WebResult<NamedFile> {
    Ok(NamedFile::open("static/comparison.htm")?)
}

#[get("/logo.svg")]
async fn serve_logo(_data: Data<AppState>) -> WebResult<NamedFile> {
    Ok(NamedFile::open("static/logo.svg")?)
}

#[derive(Deserialize)]
struct QueryRequest {
    cooccurrences: Option<bool>,
    users: String,
    threshold: Option<usize>,
    order: Option<Order>,
}

#[get("/query")]
async fn serve_query(query_request: Query<QueryRequest>, data: Data<AppState>) -> impl Responder {
    let users = query_request.users.split(',').map(|user| user.to_string()).collect();
    let ram_index = data.ram_index.lock().unwrap();
    if ram_index.data.is_empty() {
        return HttpResponse::ServiceUnavailable().body("Index not yet available (try again later)\n");
    }
    let mut cursor = Cursor::new(&*ram_index.data);
    let mut response = vec![];
    match query(&mut cursor, &mut response, &users, query_request.threshold.unwrap_or(0), query_request.order.unwrap_or(Order::none), query_request.cooccurrences.unwrap_or(false), false) {
        Ok(()) => (),
        Err(()) => (),
    }
    HttpResponse::Ok()
        .insert_header(ContentType(TEXT_PLAIN_UTF_8))
        .body(response)
}

fn get_index_name(path: &str) -> Option<String> {
    Some(Path::new(&path)
        .canonicalize().ok()?
        .file_stem().unwrap()
        .to_str().unwrap()
        .to_string())
}

fn load_index(data: &Data<AppState>) -> Result<(), &'static str> {
    let mut ram_index_data = vec![];
    let start = Instant::now();
    let name = get_index_name(&data.index_path).unwrap_or("unknown".to_string());
    if name == *data.ram_index.lock().unwrap().name {
        return Err("Index already up-to-date, no need to reload");
    }
    println!("Loading index...");
    let input = File::open(&data.index_path);
    match input {
        Ok(mut input) => {
            input.read_to_end(&mut ram_index_data).unwrap();
            let duration = start.elapsed();
            println!("Index loaded in {:?}", duration);
            let mut app_ram_index = data.ram_index.lock().unwrap();
            *app_ram_index = RamIndex {
                name: name,
                data: ram_index_data,
            };
            Ok(())
        },
        Err(error) => {
            eprintln!("socksfinder: can't open index: {}: {}", &data.index_path, &error);
            return Err("Unable to open index");
        }
    }
}

#[get("/reload")]
async fn serve_reload(data: Data<AppState>) -> impl Responder {
    match load_index(&data) {
        Ok(()) => {
            HttpResponse::Ok().body(format!("Index reloaded\n"))
        }
        Err(error) => {
            HttpResponse::InternalServerError().body(format!("{}\n", error))
        }
    }
}

#[get("/version")]
async fn serve_version(data: Data<AppState>) -> impl Responder {
    let ram_index = data.ram_index.lock().unwrap();
    HttpResponse::Ok().body(format!("Running socksfinder v{} ({})\n", version(), &ram_index.name))
}

#[actix_web::main]
pub async fn serve(index: String, hostname: String, port: u16) -> std::io::Result<()> {
    let data = Data::new(AppState {
        index_path: index,
        ram_index: Mutex::new(RamIndex {
            name: "no index".to_string(),
            data: vec![],
        }),
    });
    let initial_data = data.clone();
    std::thread::spawn(move || {
        match load_index(&initial_data) {
            Ok(()) => (),
            Err(_) => (), // index not loaded, but this can be done later using /reload
        }
    });
    println!("Listening on {}:{}...", hostname, port);
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(serve_index)
            .service(serve_badge)
            .service(serve_comparison)
            .service(serve_logo)
            .service(serve_query)
            .service(serve_reload)
            .service(serve_version)
    })
        .bind(format!("{}:{}", hostname, port))?
        .run()
        .await
}

fn print_size(label: &str, size: u64) {
    match NumberPrefix::binary(size as f64) {
        NumberPrefix::Standalone(number) => println!("{}: {} B", label, number.round()),
        NumberPrefix::Prefixed(prefix, number) => println!("{}: {} {}B", label, number.round(), prefix),
    }
}

pub fn stats(index: &mut dyn Index) -> Result<(), ()> {
    let (fst_start_offset, fst_end_offset) = read_index_header(index)?;
    index.seek(SeekFrom::Start(fst_start_offset)).unwrap();
    let mut fst_reader = index.take(fst_end_offset - fst_start_offset);
    let mut fst_bytes = vec![];
    fst_reader.read_to_end(&mut fst_bytes).unwrap();
    let fst = fst::Map::new(fst_bytes).unwrap();
    let mut stream = fst.values();
    let mut user_contribs_offset = fst_start_offset;
    let mut edit_count_range_counts = vec![0; 10];
    while let Some(value) = stream.next() {
        let edit_count = value & 0xFF_FF_FF_FF;
        if edit_count > 256 {
            edit_count_range_counts[9] += 1;
        } else {
            edit_count_range_counts[(edit_count as f64).log2().ceil() as usize] += 1;
        }
        let page_offsets_offset = value >> 32;
        if page_offsets_offset < user_contribs_offset {
            user_contribs_offset = page_offsets_offset;
        }
    }
    println!("==[ Header info ]==");
    println!("Index format version number: {}", SF_VERSION);
    println!("==[ Section offsets ]==");
    println!("Page names: 4");
    println!("User contribs: {}", user_contribs_offset);
    println!("FST: {}", fst_start_offset);
    println!("Footer: {}", fst_end_offset);
    println!("==[ Section sizes ]==");
    print_size("Header", 4);
    print_size("Page names", user_contribs_offset - 4);
    print_size("User contribs", fst_start_offset - user_contribs_offset);
    print_size("FST", fst_end_offset - fst_start_offset);
    print_size("Footer", 4);
    println!("==[ Contrib list length ]==");
    for bin in 0..9 {
        println!("]{};{}]: {}", if bin == 0 { 0 } else { 1 << (bin - 1) }, 1 << bin, edit_count_range_counts[bin]);
    }
    println!("]256;+∞[ : {}", edit_count_range_counts[9]);
    Ok(())
}
