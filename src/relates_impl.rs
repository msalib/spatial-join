impl Relates<Line<f64>> for Point<f64> {
    fn Contains(&self, _other: &Line<f64>) -> bool {
        false
    }
    fn EuclideanDistance(&self, other: &Line<f64>) -> f64 {
        self.euclidean_distance(other)
    }
    fn Intersects(&self, other: &Line<f64>) -> bool {
        self.intersects(other)
    }
}

impl Relates<LineString<f64>> for Point<f64> {
    fn Contains(&self, _other: &LineString<f64>) -> bool {
        false
    }
    fn EuclideanDistance(&self, other: &LineString<f64>) -> f64 {
        self.euclidean_distance(other)
    }
    fn Intersects(&self, other: &LineString<f64>) -> bool {
        other.Intersects(self)
    }
}

impl Relates<Polygon<f64>> for Point<f64> {
    fn Contains(&self, _other: &Polygon<f64>) -> bool {
        false
    }
    fn EuclideanDistance(&self, other: &Polygon<f64>) -> f64 {
        self.euclidean_distance(other)
    }
    fn Intersects(&self, other: &Polygon<f64>) -> bool {
        other.Intersects(self)
    }
}

impl Relates<Rect<f64>> for Point<f64> {
    fn Contains(&self, _other: &Rect<f64>) -> bool {
        false
    }
    fn EuclideanDistance(&self, other: &Rect<f64>) -> f64 {
        other.EuclideanDistance(self)
    }
    fn Intersects(&self, other: &Rect<f64>) -> bool {
        other.Intersects(self)
    }
}

impl Relates<Triangle<f64>> for Point<f64> {
    fn Contains(&self, _other: &Triangle<f64>) -> bool {
        false
    }
    fn EuclideanDistance(&self, other: &Triangle<f64>) -> f64 {
        other.EuclideanDistance(self)
    }
    fn Intersects(&self, other: &Triangle<f64>) -> bool {
        other.Intersects(self)
    }
}

impl Relates<Polygon<f64>> for Line<f64> {
    fn Contains(&self, _other: &Polygon<f64>) -> bool {
        false
    }
    fn EuclideanDistance(&self, other: &Polygon<f64>) -> f64 {
        other.EuclideanDistance(self)
    }
    fn Intersects(&self, other: &Polygon<f64>) -> bool {
        self.intersects(other)
    }
}

impl Relates<Rect<f64>> for Line<f64> {
    fn Contains(&self, _other: &Rect<f64>) -> bool {
        false
    }
    fn EuclideanDistance(&self, other: &Rect<f64>) -> f64 {
        other.EuclideanDistance(self)
    }
    fn Intersects(&self, other: &Rect<f64>) -> bool {
        other.Intersects(self)
    }
}

impl Relates<Triangle<f64>> for Line<f64> {
    fn Contains(&self, _other: &Triangle<f64>) -> bool {
        false
    }
    fn EuclideanDistance(&self, other: &Triangle<f64>) -> f64 {
        other.EuclideanDistance(self)
    }
    fn Intersects(&self, other: &Triangle<f64>) -> bool {
        other.Intersects(self)
    }
}

impl Relates<Polygon<f64>> for LineString<f64> {
    fn Contains(&self, _other: &Polygon<f64>) -> bool {
        false
    }
    fn EuclideanDistance(&self, other: &Polygon<f64>) -> f64 {
        self.euclidean_distance(other)
    }
    fn Intersects(&self, other: &Polygon<f64>) -> bool {
        self.intersects(other)
    }
}

impl Relates<Rect<f64>> for LineString<f64> {
    fn Contains(&self, _other: &Rect<f64>) -> bool {
        false
    }
    fn EuclideanDistance(&self, other: &Rect<f64>) -> f64 {
        other.EuclideanDistance(self)
    }
    fn Intersects(&self, other: &Rect<f64>) -> bool {
        other.Intersects(self)
    }
}

impl Relates<Triangle<f64>> for LineString<f64> {
    fn Contains(&self, _other: &Triangle<f64>) -> bool {
        false
    }
    fn EuclideanDistance(&self, other: &Triangle<f64>) -> f64 {
        other.EuclideanDistance(self)
    }
    fn Intersects(&self, other: &Triangle<f64>) -> bool {
        other.Intersects(self)
    }
}

impl Relates<LineString<f64>> for LineString<f64> {
    fn Contains(&self, other: &LineString<f64>) -> bool {
        other
            .lines()
            .all(|oline| self.lines().any(|sline| sline.Contains(&oline)))
    }
    fn EuclideanDistance(&self, other: &LineString<f64>) -> f64 {
        self.euclidean_distance(other)
    }
    fn Intersects(&self, other: &LineString<f64>) -> bool {
        self.intersects(other)
    }
}

impl Relates<Line<f64>> for Rect<f64> {
    fn Contains(&self, other: &Line<f64>) -> bool {
        self.contains(&other.start_point()) && self.contains(&other.end_point())
    }
    fn EuclideanDistance(&self, other: &Line<f64>) -> f64 {
        if self.Intersects(other) {
            0.0
        } else {
            rect_lines(self)
                .iter()
                .map(|sline| sline.euclidean_distance(other))
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
        }
    }
    fn Intersects(&self, other: &Line<f64>) -> bool {
        self.Contains(other) || rect_lines(self).iter().any(|sline| sline.intersects(other))
    }
}

impl Relates<LineString<f64>> for Rect<f64> {
    fn Contains(&self, other: &LineString<f64>) -> bool {
        other.points_iter().all(|pt| self.Contains(&pt))
    }
    fn EuclideanDistance(&self, other: &LineString<f64>) -> f64 {
        if self.Intersects(other) {
            0.0
        } else {
            rect_lines(self)
                .iter()
                .map(|sline| sline.euclidean_distance(other))
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
        }
    }
    fn Intersects(&self, other: &LineString<f64>) -> bool {
        self.Contains(other)
            || rect_lines(self)
                .iter()
                .any(|sline| other.lines().any(|oline| sline.intersects(&oline)))
    }
}

impl Relates<Triangle<f64>> for Rect<f64> {
    fn Contains(&self, other: &Triangle<f64>) -> bool {
        self.contains(&Point(other.0))
            && self.contains(&Point(other.1))
            && self.contains(&Point(other.2))
    }
    fn EuclideanDistance(&self, other: &Triangle<f64>) -> f64 {
        if self.Intersects(other) {
            0.0
        } else {
            rect_lines(self)
                .iter()
                .map(|sline| sline.EuclideanDistance(other))
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
        }
    }
    fn Intersects(&self, other: &Triangle<f64>) -> bool {
        self.Contains(other)
            || rect_lines(self)
                .iter()
                .any(|sline| other.to_lines().iter().any(|oline| sline.intersects(oline)))
            || other.Contains(self)
    }
}

impl Relates<Polygon<f64>> for Rect<f64> {
    fn Contains(&self, other: &Polygon<f64>) -> bool {
        other.exterior().points_iter().all(|pt| self.Contains(&pt))
    }
    fn EuclideanDistance(&self, other: &Polygon<f64>) -> f64 {
        if self.Intersects(other.exterior()) {
            0.0
        } else {
            rect_lines(self)
                .iter()
                .map(|sline| sline.EuclideanDistance(other))
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
        }
    }
    fn Intersects(&self, other: &Polygon<f64>) -> bool {
        self.intersects(other)
    }
}

impl Relates<Point<f64>> for Triangle<f64> {
    fn Contains(&self, other: &Point<f64>) -> bool {
        if self.0 == self.1 && self.1 == self.2 {
            self.0 == other.0
        } else {
            self.contains(other)
        }
    }
    fn EuclideanDistance(&self, other: &Point<f64>) -> f64 {
        if self.Intersects(other) {
            0.0
        } else {
            self.to_lines()
                .iter()
                .map(|sline| sline.euclidean_distance(other))
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
        }
    }
    fn Intersects(&self, other: &Point<f64>) -> bool {
        self.Contains(other)
    }
}

impl Relates<Line<f64>> for Triangle<f64> {
    fn Contains(&self, other: &Line<f64>) -> bool {
        self.Contains(&other.start_point()) && self.Contains(&other.end_point())
    }
    fn EuclideanDistance(&self, other: &Line<f64>) -> f64 {
        if self.Intersects(other) {
            0.0
        } else {
            self.to_lines()
                .iter()
                .map(|sline| sline.euclidean_distance(other))
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
        }
    }
    fn Intersects(&self, other: &Line<f64>) -> bool {
        self.Contains(other) || self.to_lines().iter().any(|sline| sline.intersects(other))
    }
}

impl Relates<LineString<f64>> for Triangle<f64> {
    fn Contains(&self, other: &LineString<f64>) -> bool {
        other.lines().all(|line| self.Contains(&line))
    }
    fn EuclideanDistance(&self, other: &LineString<f64>) -> f64 {
        if self.Intersects(other) {
            0.0
        } else {
            self.to_lines()
                .iter()
                .map(|sline| {
                    other
                        .lines()
                        .map(|oline| oline.euclidean_distance(sline))
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                })
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
                .unwrap()
        }
    }
    fn Intersects(&self, other: &LineString<f64>) -> bool {
        self.Contains(other)
            || self
                .to_lines()
                .iter()
                .any(|sline| other.lines().any(|oline| sline.intersects(&oline)))
    }
}

impl Relates<Triangle<f64>> for Triangle<f64> {
    fn Contains(&self, other: &Triangle<f64>) -> bool {
        self.Contains(&Point(other.0))
            && self.Contains(&Point(other.1))
            && self.Contains(&Point(other.2))
    }
    fn EuclideanDistance(&self, other: &Triangle<f64>) -> f64 {
        if self.Intersects(other) {
            0.0
        } else {
            other
                .to_lines()
                .iter()
                .map(|oline| {
                    self.to_lines()
                        .iter()
                        .map(|sline| sline.euclidean_distance(oline))
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                })
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
                .unwrap()
        }
    }
    fn Intersects(&self, other: &Triangle<f64>) -> bool {
        self.Contains(other)
            || self
                .to_lines()
                .iter()
                .any(|sline| other.to_lines().iter().any(|oline| sline.intersects(oline)))
            || other.Contains(self)
    }
}

impl Relates<Polygon<f64>> for Triangle<f64> {
    fn Contains(&self, other: &Polygon<f64>) -> bool {
        other.exterior().points_iter().all(|pt| self.Contains(&pt))
    }
    fn EuclideanDistance(&self, other: &Polygon<f64>) -> f64 {
        if self.Intersects(other.exterior()) {
            0.0
        } else {
            self.to_lines()
                .iter()
                .map(|sline| sline.EuclideanDistance(other))
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
        }
    }
    fn Intersects(&self, other: &Polygon<f64>) -> bool {
        self.Intersects(other.exterior())
            || (other.exterior().Contains(self)
                || if other.interiors().is_empty() {
                    false
                } else {
                    other.interiors().iter().all(|hole| !hole.Contains(self))
                })
    }
}

impl Relates<Rect<f64>> for Triangle<f64> {
    fn Contains(&self, other: &Rect<f64>) -> bool {
        rect_lines(other).iter().all(|line| self.Contains(line))
    }
    fn EuclideanDistance(&self, other: &Rect<f64>) -> f64 {
        other.EuclideanDistance(self)
    }
    fn Intersects(&self, other: &Rect<f64>) -> bool {
        other.Intersects(self)
    }
}

impl Relates<Rect<f64>> for Polygon<f64> {
    fn Contains(&self, other: &Rect<f64>) -> bool {
        rect_lines(other).iter().all(|line| self.contains(line))
    }
    fn EuclideanDistance(&self, other: &Rect<f64>) -> f64 {
        other.EuclideanDistance(self)
    }
    fn Intersects(&self, other: &Rect<f64>) -> bool {
        self.intersects(other)
    }
}

impl Relates<Triangle<f64>> for Polygon<f64> {
    fn Contains(&self, other: &Triangle<f64>) -> bool {
        self.contains(&Point(other.0))
            && self.contains(&Point(other.1))
            && self.contains(&Point(other.2))
    }
    fn EuclideanDistance(&self, other: &Triangle<f64>) -> f64 {
        other.EuclideanDistance(self)
    }
    fn Intersects(&self, other: &Triangle<f64>) -> bool {
        other.Intersects(self)
    }
}

impl Relates<Point<f64>> for Point<f64> {
    fn Contains(&self, other: &Point<f64>) -> bool {
        self.contains(other)
    }
    fn EuclideanDistance(&self, other: &Point<f64>) -> f64 {
        self.euclidean_distance(other)
    }
    fn Intersects(&self, other: &Point<f64>) -> bool {
        self == other
    }
}

impl Relates<Point<f64>> for Line<f64> {
    fn Contains(&self, other: &Point<f64>) -> bool {
        self.contains(other)
    }
    fn EuclideanDistance(&self, other: &Point<f64>) -> f64 {
        self.euclidean_distance(other)
    }
    fn Intersects(&self, other: &Point<f64>) -> bool {
        self.intersects(other)
    }
}

impl Relates<Line<f64>> for Line<f64> {
    fn Contains(&self, other: &Line<f64>) -> bool {
        self.contains(other)
    }
    fn EuclideanDistance(&self, other: &Line<f64>) -> f64 {
        self.euclidean_distance(other)
    }
    fn Intersects(&self, other: &Line<f64>) -> bool {
        self.intersects(other)
    }
}

impl Relates<LineString<f64>> for Line<f64> {
    fn Contains(&self, other: &LineString<f64>) -> bool {
        self.contains(other)
    }
    fn EuclideanDistance(&self, other: &LineString<f64>) -> f64 {
        self.euclidean_distance(other)
    }
    fn Intersects(&self, other: &LineString<f64>) -> bool {
        self.intersects(other)
    }
}

impl Relates<Point<f64>> for LineString<f64> {
    fn Contains(&self, other: &Point<f64>) -> bool {
        self.contains(other)
    }
    fn EuclideanDistance(&self, other: &Point<f64>) -> f64 {
        self.euclidean_distance(other)
    }
    fn Intersects(&self, other: &Point<f64>) -> bool {
        self.contains(other)
    }
}

impl Relates<Line<f64>> for LineString<f64> {
    fn Contains(&self, other: &Line<f64>) -> bool {
        self.contains(other)
    }
    fn EuclideanDistance(&self, other: &Line<f64>) -> f64 {
        self.euclidean_distance(other)
    }
    fn Intersects(&self, other: &Line<f64>) -> bool {
        self.intersects(other)
    }
}

impl Relates<Point<f64>> for Polygon<f64> {
    fn Contains(&self, other: &Point<f64>) -> bool {
        self.contains(other)
    }
    fn EuclideanDistance(&self, other: &Point<f64>) -> f64 {
        self.euclidean_distance(other)
    }
    fn Intersects(&self, other: &Point<f64>) -> bool {
        self.contains(other)
    }
}

impl Relates<Line<f64>> for Polygon<f64> {
    fn Contains(&self, other: &Line<f64>) -> bool {
        self.contains(other)
    }
    fn EuclideanDistance(&self, other: &Line<f64>) -> f64 {
        if self.intersects(other) {
            0.0
        } else {
            self.exterior()
                .lines()
                .map(|sline| sline.euclidean_distance(other))
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
        }
    }
    fn Intersects(&self, other: &Line<f64>) -> bool {
        self.intersects(other)
    }
}

impl Relates<LineString<f64>> for Polygon<f64> {
    fn Contains(&self, other: &LineString<f64>) -> bool {
        self.contains(other)
    }
    fn EuclideanDistance(&self, other: &LineString<f64>) -> f64 {
        self.euclidean_distance(other)
    }
    fn Intersects(&self, other: &LineString<f64>) -> bool {
        self.intersects(other)
    }
}

impl Relates<Polygon<f64>> for Polygon<f64> {
    fn Contains(&self, other: &Polygon<f64>) -> bool {
        self.contains(other)
    }
    fn EuclideanDistance(&self, other: &Polygon<f64>) -> f64 {
        self.euclidean_distance(other)
    }
    fn Intersects(&self, other: &Polygon<f64>) -> bool {
        self.intersects(other)
    }
}

impl Relates<Point<f64>> for Rect<f64> {
    fn Contains(&self, other: &Point<f64>) -> bool {
        self.contains(other)
    }
    fn EuclideanDistance(&self, other: &Point<f64>) -> f64 {
        if self.Intersects(other) {
            0.0
        } else {
            rect_lines(self)
                .iter()
                .map(|sline| sline.euclidean_distance(other))
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
        }
    }
    fn Intersects(&self, other: &Point<f64>) -> bool {
        self.Contains(other)
    }
}

impl Relates<Rect<f64>> for Rect<f64> {
    fn Contains(&self, other: &Rect<f64>) -> bool {
        self.contains(other)
    }
    fn EuclideanDistance(&self, other: &Rect<f64>) -> f64 {
        if self.Intersects(other) {
            0.0
        } else {
            rect_lines(other)
                .iter()
                .map(|oline| oline.EuclideanDistance(self))
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
        }
    }
    fn Intersects(&self, other: &Rect<f64>) -> bool {
        self.intersects(other)
    }
}
