extern crate schemafy;

use std::io::{Read, Write};
use std::process::Command;

fn main() -> Result<(), std::io::Error> {
    let schema = "src/annotation_schema.json";
    println!("cargo:rerun-if-changed={}", schema);
    let src = std::path::Path::new(schema);

    let mut file = std::fs::File::open(src).unwrap();
    let mut input = String::new();
    file.read_to_string(&mut input).unwrap();

    let output = schemafy::GenerateBuilder {
        root_name: Some("PaperAnnotations"),
        schemafy_path: "::",
        rustfmt_cmd: Some(Command::new("rustfmt")),
    }
    .build(&input)
    .unwrap();
    let dst = std::path::Path::new("src/schema.rs");

    let mut file = std::fs::File::create(dst).unwrap();
    file.write_all(
        br#"
                   use serde::{Serialize, Deserialize};
                   "#,
    )
    .unwrap();
    file.write_all(output.as_bytes())
}
