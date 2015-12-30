extern crate rustc_serialize;
extern crate csv;
#[macro_use] extern crate la;

use std::fs;
use std::error::Error;
use la::*;

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


fn main() {

    let filename = "data.csv";

    let data = match parse_file(filename) {
            Ok(data) => data,
            Err(err)  => {
                println!("Problem reading file {}: {}", filename, err.to_string());
                std::process::exit(1)
            },
        };

    println!("data is now {:?}", data);

    let x1 = data.get(0, 0);
    let y1 = data.get(0, 1);

    let x2 = data.get(1, 0);
    let y2 = data.get(1, 1);

    let l = calc_line(x1, y1, x2, y2);

    println!("The line has m = {} and c = {}", l.m, l.c);
}


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


