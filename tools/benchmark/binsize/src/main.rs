// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use std::fs;
use std::string::String;

fn wasm_filesize(dir: &str) -> Result<u64, std::io::Error> {
    let paths = fs::read_dir(dir).expect("Directory wasm/ not found!");
    for path in paths {
        let p = path.unwrap().path();
        if let Some(suffix) = p.extension() {
            if suffix == "wasm" {
                println!(
                    "test {} ... bench:      {} ns/iter (+/- 0)",
                    String::from(p.file_stem().unwrap().to_str().unwrap()),
                    p.metadata()?.len()
                );
            }
        }
    }
    Ok(1)
}

fn main() {
    let _rc = wasm_filesize("wasmpkg");
}
