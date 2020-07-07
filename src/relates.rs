use geo::algorithm::contains::Contains;
use geo::algorithm::euclidean_distance::EuclideanDistance;
use geo::algorithm::intersects::Intersects;
use geo::{Coordinate, Line, LineString, Point, Polygon, Rect, Triangle};

// Generated with gen.py; too bad rust macros aren't powerful enough
// to handle this sort of thing without making another package for a
// proc-macro.

// rename Interaction to Relation and put it along with this trait +impls into a relations module
#[allow(non_snake_case)]
pub trait Relates<T> {
    // FIXME: explain why we use CamelCase
    fn Contains(&self, other: &T) -> bool;
    fn Intersects(&self, other: &T) -> bool;
    fn EuclideanDistance(&self, other: &T) -> f64;
}

#[allow(clippy::many_single_char_names)]
fn rect_points(r: &Rect<f64>) -> [Coordinate<f64>; 4] {
    // These points are arranged in clockwise order:
    // b              c
    //  +------------+
    //  |            |
    //  |            |
    //  |            |
    //  |            |
    //  |            |
    //  +------------+
    // a              d
    let a = r.min();
    let c = r.max();
    let b: Coordinate<f64> = (a.x, c.y).into();
    let d: Coordinate<f64> = (c.x, a.y).into();
    [a, b, c, d]
}

#[allow(clippy::many_single_char_names)]
fn rect_lines(r: &Rect<f64>) -> [Line<f64>; 4] {
    let [a, b, c, d] = rect_points(r);
    [
        Line::new(a, b),
        Line::new(b, c),
        Line::new(c, d),
        Line::new(d, a),
    ]
}

include!("relates_impl.rs");
