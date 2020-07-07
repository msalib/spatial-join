use geo::algorithm::bounding_rect::BoundingRect;
use geo::{Line, LineString, Point, Polygon, Rect, Triangle};
use rstar::RTree;

use crate::SplitGeoSeq;

type RTreeEnvelope = rstar::AABB<[f64; 2]>;

#[derive(Debug)]
pub struct FakeRegion {
    pub id: usize,
    pub bbox: RTreeEnvelope,
}

impl rstar::RTreeObject for FakeRegion {
    type Envelope = RTreeEnvelope;

    fn envelope(&self) -> Self::Envelope {
        self.bbox
    }
}

impl SplitGeoSeq {
    pub fn to_rtrees(&self, max_distance: f64) -> [RTree<FakeRegion>; 6] {
        // Why duplicate? Because bounding_rect isn't defined for
        // Point and for the geos it is defined for, it sometimes
        // gives you a Rect and sometimes Option<Rect>
        [
            RTree::bulk_load(
                self.geos
                    .points
                    .iter()
                    .enumerate()
                    .map(|(index, pt)| FakeRegion {
                        id: index,
                        bbox: cheap_buffer(pt.to_env(), max_distance),
                    })
                    .collect(),
            ),
            RTree::bulk_load(
                self.geos
                    .lines
                    .iter()
                    .enumerate()
                    .map(|(index, ln)| FakeRegion {
                        id: index,
                        bbox: cheap_buffer(ln.to_env(), max_distance),
                    })
                    .collect(),
            ),
            RTree::bulk_load(
                self.geos
                    .polys
                    .iter()
                    .enumerate()
                    .map(|(index, poly)| FakeRegion {
                        id: index,
                        bbox: cheap_buffer(poly.to_env(), max_distance),
                    })
                    .collect(),
            ),
            RTree::bulk_load(
                self.geos
                    .line_strings
                    .iter()
                    .enumerate()
                    .map(|(index, ls)| FakeRegion {
                        id: index,
                        bbox: cheap_buffer(ls.to_env(), max_distance),
                    })
                    .collect(),
            ),
            RTree::bulk_load(
                self.geos
                    .rects
                    .iter()
                    .enumerate()
                    .map(|(index, rect)| FakeRegion {
                        id: index,
                        bbox: cheap_buffer(rect.to_env(), max_distance),
                    })
                    .collect(),
            ),
            RTree::bulk_load(
                self.geos
                    .tris
                    .iter()
                    .enumerate()
                    .map(|(index, tri)| FakeRegion {
                        id: index,
                        bbox: cheap_buffer(tri.to_env(), max_distance),
                    })
                    .collect(),
            ),
        ]
    }
}

pub trait Envelope {
    fn to_env(&self) -> RTreeEnvelope;
}

impl Envelope for Point<f64> {
    fn to_env(&self) -> RTreeEnvelope {
        RTreeEnvelope::from_point([self.x(), self.y()])
    }
}

impl Envelope for Line<f64> {
    fn to_env(&self) -> RTreeEnvelope {
        let bounds = self.bounding_rect();
        RTreeEnvelope::from_corners(
            [bounds.min().x, bounds.min().y],
            [bounds.max().x, bounds.max().y],
        )
    }
}

impl Envelope for Polygon<f64> {
    fn to_env(&self) -> RTreeEnvelope {
        let bounds = self
            .bounding_rect()
            .expect("invalid bounding_rect for Polygon");
        RTreeEnvelope::from_corners(
            [bounds.min().x, bounds.min().y],
            [bounds.max().x, bounds.max().y],
        )
    }
}

impl Envelope for LineString<f64> {
    fn to_env(&self) -> RTreeEnvelope {
        let bounds = self
            .bounding_rect()
            .expect("invalid bounding_rect for LineString");
        RTreeEnvelope::from_corners(
            [bounds.min().x, bounds.min().y],
            [bounds.max().x, bounds.max().y],
        )
    }
}

impl Envelope for Rect<f64> {
    fn to_env(&self) -> RTreeEnvelope {
        RTreeEnvelope::from_corners([self.min().x, self.min().y], [self.max().x, self.max().y])
    }
}

impl Envelope for Triangle<f64> {
    fn to_env(&self) -> RTreeEnvelope {
        let bounds = self.bounding_rect();
        RTreeEnvelope::from_corners(
            [bounds.min().x, bounds.min().y],
            [bounds.max().x, bounds.max().y],
        )
    }
}

pub fn cheap_buffer(bbox: RTreeEnvelope, distance: f64) -> RTreeEnvelope {
    let lower = bbox.lower();
    let upper = bbox.upper();
    RTreeEnvelope::from_corners(
        [lower[0] - distance, lower[1] - distance],
        [upper[0] + distance, upper[1] + distance],
    )
}
