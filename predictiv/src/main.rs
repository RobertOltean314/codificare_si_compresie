use std::path::Path;

pub fn read_image(path: &Path) -> Result<(usize, usize, Vec<Vec<u8>>), bmp::BmpError> {
    let img = bmp::open(path)?;
    let width = img.get_width() as usize;
    let height = img.get_height() as usize;
    let mut original_image: Vec<Vec<u8>> = Vec::with_capacity(height);
    for y in 0..height {
        let mut row = Vec::with_capacity(width);
        for x in 0..width {
            let pixel = img.get_pixel(x as u32, y as u32);
            let gray = pixel.r;
            row.push(gray);
        }
        original_image.push(row);
    }
    Ok((width, height, original_image))
}

fn main() -> Result<(), bmp::BmpError> {
    let path = Path::new("../Lenna256an.bmp");
    let (width, height, original_image) = read_image(path)?;

    let mut predicted_image = vec![vec![0i32; width]; height];

    predicted_image[0][0] = 128;

    for j in 1..width {
        predicted_image[0][j] = original_image[0][j - 1] as i32;
    }

    for i in 1..height {
        predicted_image[i][0] = original_image[i - 1][0] as i32;
    }

    for i in 1..height {
        for j in 1..width {
            let a = original_image[i][j - 1] as i32;
            let b = original_image[i - 1][j] as i32;
            let c = original_image[i - 1][j - 1] as i32;
            predicted_image[i][j] = a + b - c;
        }
    }

    for row in &predicted_image {
        for &pixel in row {
            print!("{:4}", pixel);
        }
        println!();
    }

    Ok(())
}
