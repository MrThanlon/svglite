use ttf_parser;
use std::{fs::read, fmt::{Display, Formatter}};

#[derive(Debug)]
struct MyError(String);

impl Display for MyError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl std::error::Error for MyError {}

struct Builder(());

impl ttf_parser::OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        println!("M {} {}", x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        println!("L {} {}", x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        println!("Q {} {} {} {}", x1, y1, x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        println!("C {} {} {} {} {} {}", x1, y1, x2, y2, x, y);
    }
    fn close(&mut self) {
        println!("Z");
    }
}

pub fn load_font(data: &[u8], c: char) -> Result<(), Box<dyn std::error::Error>> {
    let font = ttf_parser::Face::parse(data, 0)?;
    let glyph_id = match font.glyph_index(c) {
        Some(id) => id,
        None => {
            return Err(Box::new(MyError("No char".into())));
        }
    };
    let mut builder = Builder(());
    let bbox = font.outline_glyph(glyph_id, &mut builder).unwrap();
    dbg!(bbox);
    Ok(())
}

#[test]
fn test() {
    let data = read("/mnt/c/Windows/Fonts/times.ttf").unwrap();
    assert!(dbg!(load_font(&data, '我')).is_ok());
}
