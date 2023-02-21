use std::{io};
use image::{io::Reader, image_dimensions, ImageBuffer, Rgb};

enum SeamDirection {
    VERTICAL, HORIZONTAL
}

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

    let (orig_width, orig_height) = match image_dimensions(&file_path) {
        Ok(dimensions) => dimensions,
        Err(e) => panic!("{e}"),
    };

    println!("image Dimensions: width: {}, height: {}", orig_width, orig_height);

    let mut new_width = String::new();
    let mut new_height = String::new();

    println!("New width:");
    io::stdin().read_line(&mut new_width).unwrap();
    new_width = new_width.trim().to_string();
    println!("New height:");
    io::stdin().read_line(&mut new_height).unwrap();
    new_height = new_height.trim().to_string();

    let orig_img = img.to_rgb8();
    // let test = orig_img.get_pixel(10, 10);
    // println!("{:?}", test);

    // let grey_img = img.to_luma8();
    // println!("{:?}", grey_img.get_pixel(10, 10));
    // println!("{:?}", image_energy);

    let num_vert_seams = orig_width - new_width.parse::<u32>().unwrap();
    let num_hori_seams = orig_height - new_height.parse::<u32>().unwrap();

    let seam_direction: SeamDirection;
    if num_vert_seams == 0 && num_hori_seams == 0 {
        println!("No seams to carve.");
        return;
    } else if num_vert_seams > 0 {
        seam_direction = SeamDirection::VERTICAL;
    } else {
        seam_direction = SeamDirection::HORIZONTAL;
    }

    let final_img = orig_img.clone();
    while num_vert_seams > 0 || num_hori_seams > 0 {
        let image_energy = calculate_image_energy(&final_img);
        let seam = calculate_seam(image_energy, &seam_direction);
    }


}

fn calculate_seam(image_energy: Vec<Vec<u8>>, seam_direction: &SeamDirection) -> Vec<u16> {

    // TODO
    vec![1,2,3]
}

fn calculate_image_energy(image: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Vec<Vec<u8>> {
    let width = image.width();
    let height = image.height();
    let mut image_energy = vec![vec![0u8; width as usize]; height as usize];

    for row_number in 0..height {

        let (top_pixel_row, bottom_pixel_row) = calculate_bounds_pixel_positions(height, row_number);

        for column_number in 0..width {

            let (left_pixel_col, right_pixel_col) = calculate_bounds_pixel_positions(width, column_number);

            image_energy[column_number as usize][row_number as usize] = calculate_pixel_energy(image.get_pixel(column_number, top_pixel_row), 
                                                                            image.get_pixel(column_number, bottom_pixel_row), 
                                                                            image.get_pixel(left_pixel_col, row_number), 
                                                                            image.get_pixel(right_pixel_col, row_number));
        }
    }

    image_energy
}

fn calculate_bounds_pixel_positions(range: u32, position: u32) -> (u32,u32) {
    let lower: u32;
    let upper: u32;
    if position < 1 {
        lower = range - 1;
        upper = position + 1;
    } else if position >= range - 1 {
        lower = position - 1;
        upper = 0;
    } else {
        lower = position - 1;
        upper = position + 1;
    }

    (lower, upper)
}

fn calculate_pixel_energy(top_pixel: &Rgb<u8>, bottom_pixel: &Rgb<u8>, left_pixel: &Rgb<u8>, right_pixel: &Rgb<u8>) -> u8 {
    let summed_gradient = calculate_pixel_gradient(top_pixel, bottom_pixel) + calculate_pixel_gradient(left_pixel, right_pixel);
    format_energy(summed_gradient)
}

fn calculate_pixel_gradient(pixel1: &Rgb<u8>, pixel2: &Rgb<u8>) -> u32 {

    let r_diff = pixel1.0[0].abs_diff(pixel2.0[0]);
    let g_diff = pixel1.0[1].abs_diff(pixel2.0[1]);
    let b_diff = pixel1.0[2].abs_diff(pixel2.0[2]);

    let r_sq = u32::pow(r_diff as u32, 2);
    let g_sq = u32::pow(g_diff as u32, 2);
    let b_sq = u32::pow(b_diff as u32, 2);

    r_sq + g_sq + b_sq
}

fn format_energy(raw_energy: u32) -> u8 {

    let max_pixel_value = 255;
    let upperbound = u32::pow(max_pixel_value, 2) * 6;
    let normalised_energy = raw_energy as f32 / upperbound as f32;

    (normalised_energy * max_pixel_value as f32).floor() as u8
}

