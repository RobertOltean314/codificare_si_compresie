use crate::{
    bit_operations::BitReader,
    models::{MyCustomImage, Predictor},
};

pub fn read_image_data(bytes: &[u8]) -> Result<MyCustomImage, image::ImageError> {
    let image = image::load_from_memory(bytes)?.to_rgb8();

    let height = image.height() as usize;
    let width = image.width() as usize;

    let mut data = Vec::with_capacity(height * width);

    for y in 0..height {
        let mut row = Vec::with_capacity(width);
        for x in 0..width {
            let pixel = image.get_pixel(x as u32, y as u32);
            row.push(pixel[0] as i32); // ([0], [1], [2]) for RGB, rigth now only Red
        }
        data.push(row);
    }

    Ok(MyCustomImage {
        height,
        width,
        data,
    })
}

pub fn compute_histogram(data: &Vec<Vec<i32>>, is_error: bool) -> [u32; 256] {
    let mut histogram = [0u32; 256];

    for row in data {
        for &val in row {
            let index = if is_error {
                ((val + 128).clamp(0, 255)) as usize
            } else {
                val.clamp(0, 255) as usize
            };
            histogram[index] += 1;
        }
    }

    histogram
}

pub fn get_prediction_matrix(image: &MyCustomImage, prediction_type: usize) -> Vec<Vec<i32>> {
    let mut predict = vec![vec![0i32; image.width]; image.height];
    predict[0][0] = 128;

    for row in 1..image.height {
        predict[row][0] = image.data[row - 1][0];
    }
    for col in 1..image.width {
        predict[0][col] = image.data[0][col - 1];
    }

    let predict_fn = get_predictor(prediction_type);

    for row in 1..image.height {
        for col in 1..image.width {
            let a = image.data[row][col - 1];
            let b = image.data[row - 1][col];
            let c = image.data[row - 1][col - 1];

            predict[row][col] = predict_fn(a, b, c);
        }
    }

    predict
}

pub fn predict(image: &MyCustomImage, prediction_type: usize) -> MyCustomImage {
    let predict = get_prediction_matrix(image, prediction_type);
    let mut error_data = vec![vec![0i32; image.width]; image.height];

    for row in 0..image.height {
        for col in 0..image.width {
            error_data[row][col] = image.data[row][col] - predict[row][col];
        }
    }

    MyCustomImage {
        height: image.height,
        width: image.width,
        data: error_data,
    }
}

pub fn get_header_data(reader: &mut BitReader) -> Result<Vec<u8>, actix_web::Error> {
    let mut header = vec![0u8; 1078];
    for i in 0..1078 {
        if let Some(byte_bits) = reader.read_n_bits(8) {
            header[i] = byte_bits as u8;
        } else {
            return Err(actix_web::error::ErrorBadRequest("Failed to read header"));
        }
    }
    Ok(header)
}

pub fn get_prediction_type(reader: &mut BitReader) -> Result<usize, actix_web::Error> {
    let prediction_type = reader
        .read_n_bits(4)
        .map(|v| v as usize)
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Failed to read prediction type"))?;

    Ok(prediction_type)
}

pub fn get_error_data(reader: &mut BitReader) -> Result<Vec<Vec<i32>>, actix_web::Error> {
    let mut error_data = vec![vec![0i32; 256]; 256];

    for y in 0..256 {
        for x in 0..256 {
            let err_byte = reader
                .read_n_bits(8)
                .ok_or_else(|| actix_web::error::ErrorBadRequest("Failed to read error data"))?
                as u8;
            let err_i8 = err_byte as i8;
            error_data[y][x] = err_i8 as i32;
        }
    }
    Ok(error_data)
}

pub fn reconstruct_image(
    error_data: &Vec<Vec<i32>>,
    prediction_type: usize,
) -> Result<Vec<Vec<i32>>, actix_web::Error> {
    let mut decoded = vec![vec![0i32; 256]; 256];

    decoded[0][0] = 128 + error_data[0][0];
    for x in 1..256 {
        decoded[0][x] = decoded[0][x - 1] + error_data[0][x];
    }
    for y in 1..256 {
        decoded[y][0] = decoded[y - 1][0] + error_data[y][0];
    }

    let predict_fn = get_predictor(prediction_type);

    for y in 1..256 {
        for x in 1..256 {
            let a = decoded[y][x - 1];
            let b = decoded[y - 1][x];
            let c = decoded[y - 1][x - 1];
            let pred = predict_fn(a, b, c);
            decoded[y][x] = pred + error_data[y][x];
        }
    }

    Ok(decoded)
}

pub fn get_predictor(prediction_type: usize) -> Predictor {
    let predictors: [Predictor; 10] = [
        |_, _, _| 128,
        |a, _, _| a,
        |_, b, _| b,
        |_, _, c| c,
        |a, b, c| a + b - c,
        |a, b, c| (a + (b - c)) / 2,
        |a, b, c| (b + (c - a)) / 2,
        |a, b, _| (a + b) / 2,
        |a, b, c| {
            let p = a + b - c;
            let pa = (p - a).abs();
            let pb = (p - b).abs();
            let pc = (p - c).abs();
            if pc <= pa && pc <= pb {
                c
            } else if pb <= pa {
                b
            } else {
                a
            }
        },
        |a, b, c| {
            let diff = (a * a - b * b - c * c).abs();
            (diff as f32).sqrt().round() as i32
        },
    ];

    predictors
        .get(prediction_type)
        .copied()
        .unwrap_or(predictors[4])
}

pub fn build_bmp(header: &[u8], decoded: &[Vec<i32>]) -> Vec<u8> {
    let mut decoded_bmp = header.to_vec();
    for y in (0..256).rev() {
        for x in 0..256 {
            let val = decoded[y][x].clamp(0, 255) as u8;
            decoded_bmp.push(val);
            decoded_bmp.push(val);
            decoded_bmp.push(val);
        }
        let padding = (3 * 256) % 4;
        if padding != 0 {
            for _ in 0..(4 - padding) {
                decoded_bmp.push(0);
            }
        }
    }
    decoded_bmp
}
