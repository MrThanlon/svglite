use jpeg_decoder::Decoder;
use std::fs::File;
use std::io::{BufReader, Write};

#[test]
fn jpeg_decode() {
    let file = File::open("case/github-mark.jpg").expect("failed to open file");
    let mut decoder = Decoder::new(BufReader::new(file));
    let pixels = decoder.decode().expect("failed to decode image");
    let metadata = decoder.info().unwrap();
    dbg!(metadata);
    let mut file = File::create("case/github-mark.bin").expect("failed to open file");
    dbg!(file.write(&pixels).unwrap());
}
