pub mod my_custom_image;
use my_custom_image::MyCustomImage;

use std::path::Path;

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
