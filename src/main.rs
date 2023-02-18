use std::io;
use image::{io::Reader, image_dimensions};

fn main() {
    println!("Image Path:");
    let mut file_path = String::new();
    io::stdin().read_line(&mut file_path).unwrap();
    file_path = file_path.trim().to_string(); // need to remove a newline character at end of user input
    println!("You entered {file_path}");

    let img_reader = match Reader::open(&file_path) {
        Ok(reader) => reader,
        Err(e) => panic!("Unable to open file: {e}")
    };
    let img  = match img_reader.decode() {
        Ok(img) => img,
        Err(_) => panic!("Unable to decode")
    };

    let dimensions = match image_dimensions(&file_path) {
        Ok(dimensions) => dimensions,
        Err(e) => panic!("{e}"),
    };

    println!("width: {}, height: {}", dimensions.0, dimensions.1);

    let orig_img = img.to_rgb8();
    println!("{:?}", orig_img.get_pixel(10, 10));

}
