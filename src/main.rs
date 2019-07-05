#![feature(async_await)]

#[macro_use]
extern crate log;

mod schema;
mod schema_impl;

use actix_web::{
    http::header::{HeaderValue, CONTENT_TYPE},
    http::StatusCode,
    web::{self, Bytes, Path},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use clap::{App as ClapApp, Arg, SubCommand};
use futures::compat::Compat01As03;
use reqwest::Client;
use rustc_hex::ToHex;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tiny_keccak::sha3_256;

use schema::PaperAnnotations;
use schema_impl::PapersByHash;

#[derive(Clone, Debug)]
pub struct ServerState {
    papers: PapersByHash,
    data_dir: PathBuf,
}

fn placeholder() {
    let args = std::env::args().collect::<Vec<_>>();
    let in_path = &args[1];

    let document = poppler::PopplerDocument::new_from_file(in_path, "").unwrap();
    println!("Pages: {}", document.get_n_pages());
    let n_pages = document.get_n_pages();
    for page_i in 0..n_pages {
        let page = document.get_page(page_i).unwrap();

        let image_surface =
            cairo::ImageSurface::create(cairo::Format::A8, 612 * 3, 792 * 3).unwrap();
        let mut context = cairo::Context::new(&image_surface);
        context.scale(3f64, 3f64);

        page.render(&mut context);

        let mut file = File::create(format!("./output/{}.png", page_i)).unwrap();
        image_surface.write_to_png(&mut file).unwrap();
    }
}

fn health() -> Result<impl Responder, ()> {
    Ok("OK")
}

fn list_papers(state: web::Data<ServerState>) -> Result<impl Responder, ()> {
    serde_json::to_string_pretty(&state.papers.keys().collect::<Vec<_>>()).map_err(|_| ())
}

fn get_paper_page(
    state: web::Data<ServerState>,
    path_info: web::Path<(String, usize)>,
) -> Result<impl Responder, ()> {
    let paper_hash = path_info.0.clone();
    let page_num = path_info.1;

    let paper = state.papers.get(&paper_hash).unwrap();
    let in_path = paper
        .subject
        .download_target_path(state.data_dir.to_str().unwrap(), None)
        .unwrap();

    let document = poppler::PopplerDocument::new_from_file(in_path, "").unwrap();

    let page = document.get_page(page_num).unwrap();

    let mut buf: Vec<u8> = vec![];
    let image_surface = cairo::ImageSurface::create(cairo::Format::A8, 612 * 3, 792 * 3).unwrap();
    let mut context = cairo::Context::new(&image_surface);
    context.scale(3f64, 3f64);
    page.render(&mut context);
    image_surface.write_to_png(&mut buf).unwrap();

    let mut response = HttpResponse::with_body(StatusCode::OK, buf.into());
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("image/png"));
    Ok(response)
}

fn run_server(in_file: &mut PaperAnnotations, data_dir: &str) {
    let state = ServerState {
        papers: in_file.papers_by_hash(),
        data_dir: data_dir.into(),
    };
    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .route("/health", web::get().to(health))
            .route("/api/v1/papers", web::get().to(list_papers))
            .route(
                "/api/v1/papers/{paper_id}/page/{page_num}",
                web::get().to(get_paper_page),
            )
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run()
    .unwrap();
}

fn read_input_file(path: &str) -> schema::PaperAnnotations {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    serde_json::from_reader(reader).unwrap()
}

/// Download all papers listed in the `in_file` to the correct destination in `data_dir` if they
/// don't exist there yet.
fn download_papers(in_file: &mut PaperAnnotations, data_dir: &str) {
    let client = Client::new();
    for annotation_group in &mut in_file.annotations {
        if annotation_group.subject.is_downloaded(data_dir, None) {
            continue;
        }

        let download_url = &annotation_group.subject.download_url;

        let mut resp = client.get(download_url).send().unwrap();
        let mut buf: Vec<u8> = vec![];
        resp.copy_to(&mut buf).unwrap();

        let hash = sha3_256(&buf);
        let hash_str: String = hash.to_hex();
        if !annotation_group.subject.verify_download_checksum(&hash_str) {
            info!("Invalid hash for download at URL {:?}", download_url);
            continue;
        }

        let target_path = annotation_group
            .subject
            .download_target_path(data_dir, Some(&hash_str))
            .unwrap();

        info!(
            "downloaded {:?} with hash {}, to path {:?}",
            download_url, hash_str, target_path
        );
        let mut file = File::create(target_path).unwrap();
        file.write_all(&buf).unwrap();

        annotation_group.subject.download_checksum_sha_3_256 = Some(hash_str);
    }

    print!("{}", serde_json::to_string_pretty(in_file).unwrap());
}

#[runtime::main]
async fn main() -> () {
    env_logger::init();

    let matches = ClapApp::new("spia")
        .subcommand(
            SubCommand::with_name("check")
                .arg(
                    Arg::with_name("input")
                        .long("input")
                        .value_name("FILE")
                        .help("Sets a input file")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("data-dir")
                        .long("data-dir")
                        .value_name("DIR")
                        .help("Sets a data directory")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("server")
                .arg(
                    Arg::with_name("input")
                        .long("input")
                        .value_name("FILE")
                        .help("Sets a input file")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("data-dir")
                        .long("data-dir")
                        .value_name("DIR")
                        .help("Sets a data directory")
                        .takes_value(true),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("check") {
        let mut input_file = read_input_file(matches.value_of("input").unwrap());
        download_papers(&mut input_file, matches.value_of("data-dir").unwrap());
    }
    if let Some(matches) = matches.subcommand_matches("server") {
        let mut input_file = read_input_file(matches.value_of("input").unwrap());
        async {
            run_server(&mut input_file, matches.value_of("data-dir").unwrap());
        }
            .await
    }
}
