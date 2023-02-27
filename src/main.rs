use std::{io, fs::File};
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

    let mut num_vert_seams = orig_width - new_width.parse::<u32>().unwrap();
    let mut num_hori_seams = orig_height - new_height.parse::<u32>().unwrap();

    let frame = gif::Frame::from_rgb(orig_width as u16, orig_height as u16, &orig_img.as_raw());
    let mut gif = File::create("images/target/result.gif").unwrap();
    let mut encoder = gif::Encoder::new(&mut gif, frame.width, frame.height, &[]).unwrap();
    encoder.write_frame(&frame).unwrap();

    let mut seam_direction: SeamDirection;
    if num_vert_seams == 0 && num_hori_seams == 0 {
        println!("No seams to carve.");
        return;
    } else if num_vert_seams > 0 {
        seam_direction = SeamDirection::VERTICAL;
    } else {
        seam_direction = SeamDirection::HORIZONTAL;
    }

    let mut final_img = orig_img.clone();
    let mut width = final_img.width();
    let mut height = final_img.height();
    while num_vert_seams > 0 || num_hori_seams > 0 {
        let image_energy = calculate_image_energy(&final_img, width, height);
        let seam = calculate_seam(image_energy.iter().map(|row| row.as_slice()).collect::<Vec<_>>().as_slice(), &seam_direction);
        colour_seam(&mut final_img, &seam, &seam_direction);
        let frame = gif::Frame::from_rgb(orig_width as u16, orig_height as u16, &final_img.as_raw());
        encoder.write_frame(&frame).unwrap();
        remove_seam(&mut final_img, &seam, &seam_direction);
        let frame = gif::Frame::from_rgb(orig_width as u16, orig_height as u16, &final_img.as_raw());
        encoder.write_frame(&frame).unwrap();

        match seam_direction {
            SeamDirection::VERTICAL => {
                num_vert_seams -= 1;
                width -= 1;
                seam_direction = SeamDirection::HORIZONTAL;
            },
            SeamDirection::HORIZONTAL => {
                num_hori_seams -= 1;
                height -= 1;
                seam_direction = SeamDirection::VERTICAL;
            },
        }
    }

    final_img.save("images/target/result.jpg").unwrap();
}

fn remove_seam(image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, seam: &[usize], seam_direction: &SeamDirection) -> () {
    let width = image.width();
    let height = image.height();
    match seam_direction {
        SeamDirection::VERTICAL => {
            for (row_num, seam_col) in seam.iter().enumerate() {
                for col in (*seam_col as u32)..(width-1) {
                    let next_pixel = image.get_pixel(col + 1, row_num as u32);
                    image.put_pixel(col, row_num as u32, *next_pixel);
                }
                image.put_pixel(width - 1 , row_num as u32, Rgb([255,255,255]));
            }
        },
        SeamDirection::HORIZONTAL => {
            for (col_num, seam_row) in seam.iter().enumerate() {
                for row in (*seam_row as u32)..(height - 1) {
                    let next_pixel = image.get_pixel(col_num as u32, row + 1);
                    image.put_pixel(col_num as u32, row, *next_pixel);
                }
                image.put_pixel(col_num as u32, height - 1, Rgb([255,255,255]));
            }
        },
    }
}

fn colour_seam(image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, seam: &[usize], seam_direction: &SeamDirection) -> () {

    match seam_direction {
        SeamDirection::VERTICAL => {
            for (row, col) in seam.iter().enumerate() {
                image.put_pixel(*col as u32, row as u32, Rgb([255,0,0]));
            }
        },
        SeamDirection::HORIZONTAL => {
            for (col, row) in seam.iter().enumerate() {
                image.put_pixel(col as u32, *row as u32, Rgb([255,0,0]));
            }
        },
    }
}

fn calculate_seam(image_energy: &[&[u8]], seam_direction: &SeamDirection) -> Vec<usize> {
    let mut rotated_matrix: Vec<Vec<u8>> = Vec::new();

    match seam_direction {
        SeamDirection::VERTICAL => {
            for row in image_energy {
                rotated_matrix.push(row.to_vec());
            }
        },
        SeamDirection::HORIZONTAL => {
            for col in (0..image_energy[0].len()).rev() {
                let mut new_row: Vec<u8> = Vec::new();
                for row in image_energy {
                    new_row.push(row[col]);
                }
                rotated_matrix.push(new_row);
            }
        },
    }

    let mut paths: Vec<Vec<(u32, Vec<usize>)>> = Vec::new();

    for (row_num, row) in rotated_matrix.iter().enumerate() {

        let mut to_add:Vec<(u32, Vec<usize>)> = Vec::new(); 
        if row_num == 0 { 
            for col_num in 0..rotated_matrix[0].len() {
                to_add.push((u32::from(rotated_matrix[0][col_num]), vec![col_num]));
            }
        } else {
            let mut temp_row: Vec<(u32, usize)> = Vec::new();

            for (col_num, pixel) in row.iter().enumerate() {
                let prev_col = get_col_num_to_update(col_num, paths[row_num - 1].as_slice());
                temp_row.push((u32::from(*pixel), prev_col));
            }
    
            for (col, (pixel_energy, prev_row_col)) in temp_row.iter().enumerate() {
                
                let selected_path = &paths[row_num - 1][*prev_row_col];
                let mut new_path = Vec::new();
                new_path.extend_from_slice(&selected_path.1);
                new_path.push(col);
                let tuple = (pixel_energy + selected_path.0, new_path);
                to_add.push(tuple);
            }
        }
        paths.push(to_add);
    }

    let mut min_col: usize = 0;
    let mut min_val: u32 = u32::MAX;

    for (col,path) in paths[paths.len() - 1].iter().enumerate() {
        if path.0 < min_val {
            min_val = path.0;
            min_col = col;
        }
    }

    paths[paths.len() - 1][min_col].1.to_vec()

}

fn get_col_num_to_update(col_num: usize, paths: &[(u32, Vec<usize>)]) -> usize {

    let prev_row_pos = get_relative_pos(col_num, paths.len());
    let min_pos: usize = get_min_energy_pos(paths, prev_row_pos);
    min_pos
}

fn get_min_energy_pos(paths: &[(u32, Vec<usize>)], positions_to_check: (usize, usize, usize)) -> usize {

    let pos_vec = vec![positions_to_check.0, positions_to_check.1, positions_to_check.2];

    let mut min_pos: usize = 0;
    let mut min_val: u32 = u32::MAX;

    for pos in pos_vec {
        if paths[pos].0 < min_val {
            min_pos = pos;
            min_val = paths[pos].0;
        }
    }

    min_pos
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

fn calculate_image_energy(image: &ImageBuffer<Rgb<u8>, Vec<u8>>, width: u32, height: u32) -> Vec<Vec<u8>> {
    let mut image_energy = vec![vec![0u8; width as usize]; height as usize];

    for row_number in 0..height {

        let (top_pixel_row, bottom_pixel_row) = calculate_bounds_pixel_positions(height, row_number);

        for column_number in 0..width {

            let (left_pixel_col, right_pixel_col) = calculate_bounds_pixel_positions(width, column_number);

            let top_pixel = image.get_pixel(column_number, top_pixel_row);
            let bottom_pixel = image.get_pixel(column_number, bottom_pixel_row);
            let left_pixel = image.get_pixel(left_pixel_col, row_number);
            let right_pixel = image.get_pixel(right_pixel_col, row_number);
            image_energy[row_number as usize][column_number as usize] = calculate_pixel_energy(top_pixel, bottom_pixel, left_pixel, right_pixel);
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

