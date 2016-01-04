extern crate rustc_serialize;
extern crate csv;
#[macro_use] extern crate la;
extern crate gnuplot;

use std::fs;
use std::env;
use std::error::Error;
use la::{Matrix, SVD};
use gnuplot::{Figure, AxesCommon, Caption, LineWidth, AutoOption};

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

fn generate_x_matrix(xs: &Matrix<f64>, order: usize) -> Matrix<f64> {
    let gen_row = {|x: &f64| (0..(order+1)).map(|i| x.powi(i as i32)).collect::<Vec<_>>() };
    let mdata = xs.get_data().iter().fold(vec![], |mut v, x| {v.extend(gen_row(x)); v} );
    Matrix::new(xs.rows(), order+1, mdata)
}

fn linear_regression(xs: &Matrix<f64>, ys: &Matrix<f64>) -> Matrix<f64> {
    let svd = SVD::new(&xs);
    let order = xs.cols()-1;

    let u = svd.get_u();
    // cut down s matrix to the expected number of rows given order (one coefficient per x)
    let s_hat = svd.get_s().filter_rows(&|_, row| { row <= order });
    let v = svd.get_v();

    let alpha = u.t() * ys;
    // "divide each alpha_j by its corresponding s_j"
    // But they are different dimensions, so manually divide each
    // alpha_j by the diagnonal s_j
    let mut mdata = vec![];
    for i in 0..(order+1) {
        mdata.push(alpha.get(i, 0) / s_hat.get(i, i));
    }
    let sinv_alpha = Matrix::new(order+1, 1, mdata);

    v * sinv_alpha
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Expected two arguments: one for data filename and one for polynomial order; found {}", args.len()-1);
        std::process::exit(1);
    }
    let filename = &args[1];
    let order = match args[2].parse::<usize>() {
        Ok(ord) => ord,
        Err(err) => {
            println!("Expected second argument to be an integer; found {} ({})", args[2], err);
            std::process::exit(1)
        }
    };

    let data = match parse_file(filename) {
        Ok(data) => data,
        Err(err) => {
            println!("Problem reading file {}: {}", filename, err.to_string());
            std::process::exit(1)
        }
    };

    // split off the last column as y values
    let ys = data.get_columns(data.cols()-1);
    // and the first column is the values of x, which need to be expanded to matrix form
    let xs = generate_x_matrix(&data.get_columns(0), order);

    println!("data is now {:?}", data);
    println!("xs is {:?}", xs);
    println!("ys is {:?}", ys);

    let betas = linear_regression(&xs, &ys);
    println!("betas is {:?}", betas);

    let line = { |x: f64| (0..(order+1)).fold(0.0, |sum, i| sum + betas.get(i, 0) * x.powi(i as i32))};

    // gnuplot
    // TODO: take smaller steps
    let min_x = 0.0;
    let max_x = data.get_columns(0).get_data().iter().fold(0.0f64, |pmax, x| x.max(pmax) ) + 1.0;
    let min_y = line(min_x);
    let max_y = line(max_x);

    let mut fig = Figure::new();
    fig.axes2d().points(data.get_columns(0).get_data(), 
                        ys.get_data(), 
                        &[Caption("Datapoints"), LineWidth(1.5)])
        .set_x_range(AutoOption::Fix(0.0), AutoOption::Auto)
        .set_y_range(AutoOption::Fix(0.0), AutoOption::Auto)
        .set_x_label("x", &[])
        .set_y_label("y", &[])
        .lines(vec![min_x, max_x], vec![min_y, max_y], &[Caption("Regression")]);
    fig.show();

}

#[cfg(test)]
mod tests {
    use la::{Matrix, SVD};

    trait ConcatableMatrix<T> {
        fn row_concat(&self, other: Matrix<T>) -> Matrix<T>;
        fn col_concat(&self, other: Matrix<T>) -> Matrix<T>;
    }

    impl<T: Copy> ConcatableMatrix<T> for Matrix<T> {
        // cb() in library
        fn row_concat(&self, other: Matrix<T>) -> Matrix<T> {
            assert!(self.cols() == other.cols());
            let mut data = vec![];
            data.extend(self.get_data());
            data.extend(other.get_data());
            let no_rows = self.rows() + other.rows();
            Matrix::new(no_rows, self.cols(), data)
        }

        // cr() in library
        fn col_concat(&self, other: Matrix<T>) -> Matrix<T> {
            assert!(self.rows() == other.rows());
            let mut data = vec![];
            for i in 0..self.rows() {
                data.extend(self.get_rows(i).get_data());
                data.extend(other.get_rows(i).get_data());
            }
            let no_cols = self.cols() + other.cols();
            Matrix::new(self.rows(), no_cols, data)
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
    fn test_get_rows() {
        let mat = m!(1, 2, 3; 4, 5, 6; 7, 8, 9);
        let indices = [1, 2];
        assert_eq!(mat.get_rows(&indices[..]), m![4, 5, 6; 7, 8, 9]);
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

    #[test]
    fn test_row_concat() {
        let mat1 = m!(1, 2; 3, 4);
        let mat2 = m!(5, 6; 7, 8);

        let mat3 = mat1.row_concat(mat2);
        assert_eq!(mat3, m!(1, 2; 3, 4; 5, 6; 7, 8));
    }

    #[test]
    #[should_panic]
    fn test_row_concat_wrong_dimensions() {
        let mat1 = m!(1, 2, 3; 4, 5, 6);
        let mat2 = m!(7, 8; 9, 10);

        let _ = mat1.row_concat(mat2);
    }

    #[test]
    fn test_col_concat() {
        let mat1 = m!(1, 2; 3, 4);
        let mat2 = m!(5, 6; 7, 8);

        let mat3 = mat1.col_concat(mat2);
        assert_eq!(mat3, m!(1, 2, 5, 6; 3, 4, 7, 8));
    }

    #[test]
    #[should_panic]
    fn test_col_concat_wrong_dimensions() {
        let mat1 = m!(1, 2; 3, 4; 5, 6);
        let mat2 = m!(7, 8; 9, 10);

        let _ = mat1.col_concat(mat2);
    }
}
