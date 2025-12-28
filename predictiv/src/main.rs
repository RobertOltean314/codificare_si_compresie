use std::{path::Path, vec};

pub struct MyCustomImage {
    height: usize,
    width: usize,
    data: Vec<Vec<i32>>,
}

impl MyCustomImage {
    pub fn read_image(path: &Path) -> Result<Self, bmp::BmpError> {
        let img = bmp::open(path)?;
        let height = img.get_height() as usize;
        let width = img.get_width() as usize;
        let mut data = Vec::with_capacity(height);

        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                let pixel = img.get_pixel(x as u32, y as u32);
                row.push(pixel.r as i32);
            }
            data.push(row);
        }

        Ok(Self {
            height,
            width,
            data,
        })
    }
    fn get_prediction_matrix(&self) -> Vec<Vec<i32>> {
        let mut predict = vec![vec![0i32; self.width]; self.height];
        predict[0][0] = 128;

        for row in 1..self.height {
            predict[row][0] = self.data[row - 1][0];
        }
        for col in 1..self.width {
            predict[0][col] = self.data[0][col - 1];
        }
        for row in 1..self.height {
            for col in 1..self.width {
                let a = self.data[row][col - 1];
                let b = self.data[row - 1][col];
                let c = self.data[row - 1][col - 1];
                predict[row][col] = a + b - c;
            }
        }
        predict
    }

    pub fn predict(&self) -> Self {
        let predict = self.get_prediction_matrix();
        let mut error_data = vec![vec![0i32; self.width]; self.height];
        for row in 0..self.height {
            for col in 0..self.width {
                error_data[row][col] = self.data[row][col] - predict[row][col];
            }
        }

        Self {
            height: self.height,
            width: self.width,
            data: error_data,
        }
    }
}

fn main() -> Result<(), bmp::BmpError> {
    let path = Path::new("../Lenna256an.bmp");
    let original_img = MyCustomImage::read_image(path)?;
    let error_img = original_img.predict();

    println!("Error image size: {}*{}", error_img.width, error_img.height);

    for row in &error_img.data {
        for &val in row {
            print!("{:6}", val);
        }
        println!();
    }
    Ok(())
}
