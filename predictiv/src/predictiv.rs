type Predictor = fn(a: i32, b: i32, c: i32) -> i32;

pub struct Predictiv {
    pub height: usize,
    pub width: usize,
    pub data: Vec<Vec<i32>>,
}

impl Predictiv {
    pub fn read_image(bytes: &[u8]) -> Result<Self, image::ImageError> {
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

        Ok(Self {
            height,
            width,
            data,
        })
    }

    fn get_prediction_matrix(&self, prediction: usize) -> Vec<Vec<i32>> {
        let mut predict = vec![vec![0i32; self.width]; self.height];
        predict[0][0] = 128;

        for row in 1..self.height {
            predict[row][0] = self.data[row - 1][0];
        }
        for col in 1..self.width {
            predict[0][col] = self.data[0][col - 1];
        }

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

        let predict_fn = predictors.get(prediction).copied().unwrap_or(predictors[4]);

        for row in 1..self.height {
            for col in 1..self.width {
                let a = self.data[row][col - 1];
                let b = self.data[row - 1][col];
                let c = self.data[row - 1][col - 1];

                predict[row][col] = predict_fn(a, b, c);
            }
        }

        predict
    }

    pub fn predict(&self, prediction_number: usize) -> Self {
        let predict = self.get_prediction_matrix(prediction_number);
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
