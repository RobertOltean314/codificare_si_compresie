use core::error;
use std::path::{self, Path};

#[derive(Debug)]
pub struct MyCustomImage {
    height: usize,
    width: usize,
    data: Vec<Vec<i32>>,
}

pub struct ImageManager;

impl ImageManager {
    pub fn read_image(&self, path: &Path) -> Result<MyCustomImage, bmp::BmpError> {
        let img = bmp::open(path)?;
        let width = img.get_width() as usize;
        let height = img.get_height() as usize;

        let mut data = Vec::with_capacity(height);

        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                let pixel = img.get_pixel(x as u32, y as u32);
                row.push(pixel.r as i32);
            }
            data.push(row);
        }

        Ok(MyCustomImage {
            width,
            height,
            data,
        })
    }

    fn get_prediction_matrix(&self, original_img: &MyCustomImage) -> Vec<Vec<i32>> {
        let mut data = vec![vec![0i32; original_img.width]; original_img.height];
        data[0][0] = 128;

        for j in 1..original_img.height {
            data[j][0] = original_img.data[j - 1][0];
        }

        for i in 1..original_img.height {
            data[0][i] = original_img.data[0][i - 1];
        }

        for i in 1..original_img.height {
            for j in 1..original_img.width {
                let a = original_img.data[i][j - 1]; // left
                let b = original_img.data[i - 1][j]; // above
                let c = original_img.data[i - 1][j - 1]; // diagonal
                data[i][j] = a + b - c;
            }
        }
        data
    }

    pub fn predict(&self, path: &Path) -> Result<MyCustomImage, bmp::BmpError> {
        let original_img = self.read_image(path)?;
        let predicted = self.get_prediction_matrix(&original_img);

        let height = original_img.height;
        let width = original_img.width;
        let original_data = &original_img.data;

        let mut error_data = vec![vec![0i32; width]; height];

        for y in 0..height {
            for x in 0..width {
                error_data[y][x] = original_data[y][x] - predicted[y][x];
            }
        }

        Ok(MyCustomImage {
            height,
            width,
            data: error_data,
        })
    }
}
fn main() -> Result<(), bmp::BmpError> {
    let path = Path::new("../Lenna256an.bmp");
    let manager = ImageManager;

    let error_img = manager.predict(path)?;

    println!("Error image size: {}×{}", error_img.width, error_img.height);

    for i in 0..error_img.height {
        for j in 0..error_img.width {
            print!("{:6}", error_img.data[i][j]);
        }
    }

    Ok(())
}

