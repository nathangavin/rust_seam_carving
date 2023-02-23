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
        let seam = calculate_seam(image_energy.iter().map(|row| row.as_slice()).collect::<Vec<_>>().as_slice(), &seam_direction);
        println!("{:?}", seam);
        break;
    }



}

fn calculate_seam(image_energy: &[&[u8]], seam_direction: &SeamDirection) -> Vec<usize> {

    // TODO implement directional seams

    let mut paths: Vec<(u32, Vec<usize>)> = Vec::new();

    for col_num in 0..image_energy[0].len() {
        paths.push((u32::from(image_energy[0][col_num]), vec![col_num]));
    }
    
    for (row_num, row) in image_energy.iter().enumerate() {
        if row_num == 0 { continue; }

        let mut temp_row: Vec<(u32, usize)> = Vec::new();

        for (col_num, pixel) in row.iter().enumerate() {
            let prev_col = get_col_num_to_update(row, col_num, &paths);
            temp_row.push((u32::from(*pixel), prev_col));
        }

        for (col, (pixel_energy, prev_row_col)) in temp_row.iter().enumerate() {
            match paths.get_mut(col) {
                Some(path) => {
                    path.0 += pixel_energy;
                    path.1.push(*prev_row_col);
                },
                None => println!("Unable to find path element."),
            };
        }

    }

    let mut min_col: usize = 0;
    let mut min_val: u32 = u32::MAX;

    for (col,path) in paths.iter().enumerate() {
        if path.0 < min_val {
            min_val = path.0;
            min_col = col;
        }
    }

    paths[min_col].1.to_vec()

}

fn get_col_num_to_update(row: &[u8], col_num: usize, paths: &[(u32, Vec<usize>)]) -> usize {

    let prev_row_pos = get_relative_pos(col_num, paths.len());
    let min_pos: usize = match find_min_index(&vec![paths[prev_row_pos.0].0, paths[prev_row_pos.1].0, paths[prev_row_pos.2].0]) {
        Some(index) => index,
        None => {
            println!("Warning: Minimum value not found. Setting index to 0.");
            0
        },
    };

    let abs_col = min_pos + col_num - 1;
    abs_col
}

fn get_relative_pos(col_num: usize, row_len: usize) -> (usize, usize, usize) {
    if col_num == 0 {
        return (row_len - 1, col_num, col_num + 1);
    } else if col_num == row_len - 1 {
        return (col_num - 1, col_num, 0);
    } else {
        return (col_num - 1, col_num, col_num + 1);
    }
}

fn find_min_index(slice: &[u32]) -> Option<usize> {
    if slice.is_empty() {
        // Return `None` if the slice is empty.
        return None;
    }
    
    // Initialize `min_index` to the first index in the slice.
    let mut min_index = 0;

    // Loop over the remaining indices in the slice.
    for i in 1..slice.len() {
        // If the value at index `i` is less than the value at index `min_index`,
        // update `min_index` to `i`.
        if slice[i] < slice[min_index] {
            min_index = i;
        }
    }

    // Return the index of the minimum value.
    Some(min_index)
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

