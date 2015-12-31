extern crate rustc_serialize;
extern crate csv;
#[macro_use] extern crate la;

use std::fs;
use std::error::Error;
use la::{Matrix, SVD};

struct Line {
    m: f64,
    c: f64,
}

fn calc_line(x1: f64, y1: f64, x2: f64, y2: f64) -> Line {
    let m = (y1 - y2) / (x1 - x2);
    let c = y1 - m * x1;

    Line { m: m, c: c }
}

fn parse_file(file: &str) -> Result<Matrix<f64>, Box<Error>> {
    let file = try!(fs::File::open(file));
    let mut reader = csv::Reader::from_reader(file).has_headers(false);

    let lines = try!(reader.decode().collect::<csv::Result<Vec<(f64, f64)>>>());

    let data = lines.iter().fold(Vec::<f64>::new(), |mut xs, l| {
        xs.push(l.0);
        xs.push(l.1);
        xs
    });

    let rows = data.len() / 2;
    let cols = 2;

    Ok(Matrix::new(rows, cols, data))
}

fn generate_xs(data: &Matrix<f64>) -> Matrix<f64> {
    // the last column is ys, but all others are xs
    let high_xs = data.filter_columns(&|_, col| { col < (data.cols()-1) });
    // add back in x^0, ie the first column should be all 1s
    let mut xs = Vec::<f64>::new();
    for r in 0..high_xs.rows() {
        xs.push(1.0);
        xs.extend(high_xs.get_rows(r).get_data());
    }
    let rows = data.rows();
    let cols = data.cols(); // removed y col and added x^0 col
    Matrix::new(rows, cols, xs)
}

fn main() {

    let filename = "data.csv";

    let data = match parse_file(filename) {
            Ok(data) => data,
            Err(err)  => {
                println!("Problem reading file {}: {}", filename, err.to_string());
                std::process::exit(1)
            },
        };

    // split off the last column as y values
    let ys = data.get_columns(data.cols()-1);
    // all others are coefficients of x
    let xs = generate_xs(&data);

    println!("data is now {:?}", data);
    println!("xs is {:?}", xs);
    println!("ys is {:?}", ys);

    let svd = SVD::new(&xs);

    println!("SVD is u = {:?}, s = {:?}, v = {:?}", svd.get_u(), svd.get_s(), svd.get_v());

    let u = svd.get_u();
    // cut down s matrix to the expected number of rows given data cols
    let s_hat = svd.get_s().filter_rows(&|_, row| { row < data.cols() });
    println!("Subset s is {:?}", s_hat);
    let v = svd.get_v();

    // "divide each alpha_j by its corresponding s_j"
    // But they are different dimensions, so manually divide each
    // alpha_j by the diagnonal s_j
    let alpha = u.t() * ys;
    let sinv_alpha = m!(alpha.get(0, 0) / s_hat.get(0, 0); alpha.get(1, 0) / s_hat.get(1, 1));
    let betas = v * sinv_alpha;

    println!("betas are {:?}", betas);

    let x1 = data.get(0, 0);
    let y1 = data.get(0, 1);

    let x2 = data.get(1, 0);
    let y2 = data.get(1, 1);

    let l = calc_line(x1, y1, x2, y2);

    println!("The line has m = {} and c = {}", l.m, l.c);


    println!("Testing matrix manipulations!");
    let mat = m!(1, 2, 3; 4, 5, 6; 7, 8, 9);
    let indices = [1, 2];
    println!("{:?}", mat.get_rows(&indices[..]));

}

#[cfg(test)]
mod tests {
    use super::calc_line;
    use la::{Matrix, SVD};

    pub struct MatrixRowIterator<'a, T: 'a> {
        index: usize,
        matrix: &'a Matrix<T>
    }

    impl<'a, T: Copy> Iterator for MatrixRowIterator<'a, T> {
        type Item = Matrix<T>;

        fn next(&mut self) -> Option<Matrix<T>> {
            if self.index < self.matrix.rows() {
                let row = self.matrix.get_rows(self.index);
                self.index += 1;
                Some(row)
            } else {
                None
            }
        }
    }

    pub struct MatrixColIterator<'a, T: 'a> {
        index: usize,
        matrix: &'a Matrix<T>
    }

    impl<'a, T: Copy> Iterator for MatrixColIterator<'a, T> {
        type Item = Matrix<T>;

        fn next(&mut self) -> Option<Matrix<T>> {
            if self.index < self.matrix.cols() {
                let col = self.matrix.get_columns(self.index);
                self.index += 1;
                Some(col)
            } else {
                None
            }
        }
    }

    trait IterableMatrix<T> {
        fn row_iter(&self) -> MatrixRowIterator<T>;
        fn col_iter(&self) -> MatrixColIterator<T>;
    }

    impl<T: Copy> IterableMatrix<T> for Matrix<T> {
        fn row_iter(&self) -> MatrixRowIterator<T> {
            MatrixRowIterator::<T> {
                index: 0,
                matrix: self
            }
        }

        fn col_iter(&self) -> MatrixColIterator<T> {
            MatrixColIterator::<T> {
                index: 0,
                matrix: self
            }
        }
    }

    macro_rules! assert_approx_eq(
        ($left: expr, $right: expr, $tolerance: expr) => ({
            let delta = ($left - $right).abs();
            if delta > $tolerance {
                panic!("assertion failed: `left ≈ right` (left: `{:?}`, right: `{:?}`, tolerance: `{:?}`)",
                $left, $right, $tolerance)
            }
        })
    );

    #[test]
    fn test_calc_line() {
        let l = calc_line(1.0, 2.0, 5.0, 4.0);

        assert_eq!(l.m, 0.5);
        assert_eq!(l.c, 1.5);
    }

    #[test]
    fn create_matrices() {
        let mat = Matrix::new(2, 2, vec![1.0, 2.0, 3.0, 4.0]);
        let mat1 = m!(1.0, 2.0, 3.0);
        let mat2 = m!(1.0, 2.0; 3.0, 4.0);

        println!("mat is {:?}", mat);
        println!("mat1 is {:?}", mat1);
        println!("mat2 is {:?}", mat2);

        assert!(mat.rows() == 2);
        assert!(mat.cols() == 2);
        assert!(mat1.rows() == 1);
        assert!(mat1.cols() == 3);
        assert!(mat2.rows() == 2);
        assert!(mat2.cols() == 2);
    }

    #[test]
    fn test_svd() {
        // Y = beta.X + e
        // where Y is the N x 1 matrix of y values
        // and X is the N x 2 matrix of x0 and x1 values
        // x0 = 1 for all datapoints
        // the expansion form is y = beta0 * x0 + beta1 * x1 + e
        let xs = m!(1.0, 1.0; 1.0, 2.0; 1.0, 3.0);
        let ys = m!(2.0; 4.0; 6.0);

        let svd = SVD::new(&xs);
        let u = svd.get_u();
        let s = svd.get_s();
        let v = svd.get_v();

        assert!((u * s * v.t()).approx_eq(&xs));

        // "divide each alpha_j by its corresponding s_j"
        // But they are different dimensions, so manually divide each
        // alpha_j by the diagnonal s_j
        assert_eq!(u.t().cols(), ys.rows());
        let alpha = u.t() * ys;

        assert_eq!(alpha.rows(), s.rows());
        let sinv_alpha = m!(alpha.get(0, 0) / s.get(0, 0); alpha.get(1, 0) / s.get(1, 1));
        assert_eq!(sinv_alpha.rows(), 2);
        assert_eq!(sinv_alpha.cols(), 1);

        assert_eq!(v.cols(), sinv_alpha.rows());
        let betas = v * sinv_alpha;
        assert_approx_eq!(betas.get(0, 0), 0.0f64, 0.0001f64);
        assert_approx_eq!(betas.get(1, 0), 2.0f64, 0.0001f64);
    }

    #[test]
    fn test_row_iter() {
        let mat = m!(1, 2; 3, 4; 5, 6);

        let mut iter = mat.row_iter();

        let row1 = iter.next();
        assert_eq!(row1, Some(m![1, 2]));
        let row2 = iter.next();
        assert_eq!(row2, Some(m![3, 4]));
        let row3 = iter.next();
        assert_eq!(row3, Some(m![5, 6]));

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_col_iter() {
        let mat = m!(1, 2; 3, 4; 5, 6);

        let mut iter = mat.col_iter();

        let col1 = iter.next();
        assert_eq!(col1, Some(m![1; 3; 5])); // column format
        let col2 = iter.next();
        assert_eq!(col2, Some(m![2; 4; 6]));

        assert_eq!(iter.next(), None);
    }
}
