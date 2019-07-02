#![feature(async_await)]

mod schema;

use actix_web::{http, web, web::Path, App, HttpRequest, HttpServer, Responder};
use clap::{App as ClapApp, Arg, SubCommand};
use futures::compat::Compat;
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

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

fn index(info: Path<(u32, String)>) -> Result<impl Responder, ()> {
    // Proceed with normal response
    Ok(format!("Hello {}! id:{}", info.1, info.0))
}

fn run_server() {
    HttpServer::new(|| App::new().route("/{id}/{name}/index.html", web::get().to(index)))
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

#[runtime::main]
async fn main() -> () {
    let matches = ClapApp::new("spia")
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
        )
        .get_matches();

    async {
        run_server();
    }
        .await
}
