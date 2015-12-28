extern crate rustc_serialize;
extern crate csv;

use std::fs;
use std::error::Error;

struct Line {
    m: f64,
    c: f64,
}

#[derive(Debug)]
struct Coord {
    x: f64,
    y: f64,
}

fn calc_line(pt1: &Coord, pt2: &Coord) -> Line {
    let m = (pt1.y - pt2.y) / (pt1.x - pt2.x);
    let c = pt1.y - m*pt1.x;

    Line { m: m, c: c }
}

fn parse_file(file: &str) -> Result<Vec<Coord>, Box<Error>> {
    let file = try!(fs::File::open(file));
    let mut reader = csv::Reader::from_reader(file).has_headers(false);

    type Row = (f64, f64);
    let rows = try!(reader.decode().collect::<csv::Result<Vec<Row>>>());

    println!("rows: {:?}", rows);

    //let mut coords = rows.map(|r| Coord { x: r.x, y: r.y });

    //println!("coords is now {:?}", coords);

    //coords
    return Err(From::from("Chickens"));
}


fn main() {

    let coords = parse_file("data.csv");

    // might not be 2 items
    let l = calc_line(&coords[0], &coords[1]);

    println!("The line has m = {} and c = {}", l.m, l.c);
}


#[test]
fn test_calc_line() {
    let p1 = Coord { x: 1.0, y: 2.0 };
    let p2 = Coord { x: 5.0, y: 4.0 };

    let l = calc_line(&p1, &p2);

    assert_eq!(l.m, 0.5);
    assert_eq!(l.c, 1.5);
}


