use geo::{Geometry, Line, LineString, Point, Polygon, Rect, Triangle};

use crate::Error;

pub(crate) trait IsSafe {
    fn is_safe(&self, position: usize) -> Result<(), Error>;
}

impl IsSafe for Point<f64> {
    fn is_safe(&self, position: usize) -> Result<(), Error> {
        if self.x().is_finite() && self.y().is_finite() {
            Ok(())
        } else {
            Err(Error::BadCoordinateValue(position, Geometry::Point(*self)))
        }
    }
}

impl IsSafe for Line<f64> {
    fn is_safe(&self, position: usize) -> Result<(), Error> {
        let r = self
            .start_point()
            .is_safe(position)
            .and(self.end_point().is_safe(position));
        if r.is_err() {
            Err(Error::BadCoordinateValue(position, Geometry::Line(*self)))
        } else {
            Ok(())
        }
        //            .map_err(|_| Box::new(Error::BadCoordinateValue((*self).into())))
    }
}

// Our Relates impls that use all() rely on the length checks for
// LineString and Polygon here since all(empty()) returns true. If we
// ever change those length checks, we have to review all uses of
// all() in Relates impls.

impl IsSafe for LineString<f64> {
    fn is_safe(&self, position: usize) -> Result<(), Error> {
        if self.0.len() < 2 {
            return Err(Error::LineStringTooSmall(position));
        }
        for pt in self.points_iter() {
            if pt.is_safe(position).is_err() {
                return Err(Error::BadCoordinateValue(
                    position,
                    Geometry::LineString(self.clone()),
                ));
            }
        }
        Ok(())
    }
}

impl IsSafe for Rect<f64> {
    fn is_safe(&self, position: usize) -> Result<(), Error> {
        let min: Point<f64> = self.min().into();
        let max: Point<f64> = self.max().into();
        let r = min.is_safe(position).and(max.is_safe(position));
        if r.is_err() {
            Err(Error::BadCoordinateValue(position, Geometry::Rect(*self)))
        } else {
            Ok(())
        }
    }
}

impl IsSafe for Triangle<f64> {
    fn is_safe(&self, position: usize) -> Result<(), Error> {
        let [a, b, c] = self.to_array();
        let a: Point<f64> = a.into();
        let b: Point<f64> = b.into();
        let c: Point<f64> = c.into();
        let r = a
            .is_safe(position)
            .and(b.is_safe(position))
            .and(c.is_safe(position));
        if r.is_err() {
            Err(Error::BadCoordinateValue(
                position,
                Geometry::Triangle(*self),
            ))
        } else {
            Ok(())
        }
    }
}

impl IsSafe for Polygon<f64> {
    fn is_safe(&self, position: usize) -> Result<(), Error> {
        if self.exterior().num_coords() < 3 {
            return Err(Error::PolygonExteriorTooSmall(position));
        }
        for line_string in std::iter::once(self.exterior()).chain(self.interiors().iter()) {
            if line_string.is_safe(position).is_err() {
                return Err(Error::BadCoordinateValue(
                    position,
                    Geometry::Polygon(self.clone()),
                ));
            }
        }
        Ok(())
    }
}

// //<T as TryInto<SplitGeoSeq>>::Error
// impl From<std::convert::Infallible> for Error {
//     fn from(t: std::convert::Infallible) -> Self {
//         Error::Sigh()
//     }
// }
