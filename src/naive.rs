use geo::Geometry;

use crate::{relates::Relates, Interaction, ProxMapRow, SJoinRow, SplitGeoSeq};

// FIXME: comment explaining that this comes from gen2.py
#[macro_export]
macro_rules! enum_dispatch {
    ($a:ident, $b:ident, $expr:expr) => {
        match ($a, $b) {
            (Geometry::Line($a), Geometry::Line($b)) => $expr,
            (Geometry::Line($a), Geometry::Point($b)) => $expr,
            (Geometry::Line($a), Geometry::Polygon($b)) => $expr,
            (Geometry::Line($a), Geometry::LineString($b)) => $expr,
            (Geometry::Line($a), Geometry::Rect($b)) => $expr,
            (Geometry::Line($a), Geometry::Triangle($b)) => $expr,
            (Geometry::Point($a), Geometry::Line($b)) => $expr,
            (Geometry::Point($a), Geometry::Point($b)) => $expr,
            (Geometry::Point($a), Geometry::Polygon($b)) => $expr,
            (Geometry::Point($a), Geometry::LineString($b)) => $expr,
            (Geometry::Point($a), Geometry::Rect($b)) => $expr,
            (Geometry::Point($a), Geometry::Triangle($b)) => $expr,
            (Geometry::Polygon($a), Geometry::Line($b)) => $expr,
            (Geometry::Polygon($a), Geometry::Point($b)) => $expr,
            (Geometry::Polygon($a), Geometry::Polygon($b)) => $expr,
            (Geometry::Polygon($a), Geometry::LineString($b)) => $expr,
            (Geometry::Polygon($a), Geometry::Rect($b)) => $expr,
            (Geometry::Polygon($a), Geometry::Triangle($b)) => $expr,
            (Geometry::LineString($a), Geometry::Line($b)) => $expr,
            (Geometry::LineString($a), Geometry::Point($b)) => $expr,
            (Geometry::LineString($a), Geometry::Polygon($b)) => $expr,
            (Geometry::LineString($a), Geometry::LineString($b)) => $expr,
            (Geometry::LineString($a), Geometry::Rect($b)) => $expr,
            (Geometry::LineString($a), Geometry::Triangle($b)) => $expr,
            (Geometry::Rect($a), Geometry::Line($b)) => $expr,
            (Geometry::Rect($a), Geometry::Point($b)) => $expr,
            (Geometry::Rect($a), Geometry::Polygon($b)) => $expr,
            (Geometry::Rect($a), Geometry::LineString($b)) => $expr,
            (Geometry::Rect($a), Geometry::Rect($b)) => $expr,
            (Geometry::Rect($a), Geometry::Triangle($b)) => $expr,
            (Geometry::Triangle($a), Geometry::Line($b)) => $expr,
            (Geometry::Triangle($a), Geometry::Point($b)) => $expr,
            (Geometry::Triangle($a), Geometry::Polygon($b)) => $expr,
            (Geometry::Triangle($a), Geometry::LineString($b)) => $expr,
            (Geometry::Triangle($a), Geometry::Rect($b)) => $expr,
            (Geometry::Triangle($a), Geometry::Triangle($b)) => $expr,
            _ => panic!("match failure in enum_dispatch!"),
        }
    };
}

pub(crate) fn slow_prox_map(
    small: &SplitGeoSeq,
    big: &SplitGeoSeq,
    max_distance: f64,
) -> Vec<ProxMapRow> {
    let mut result = Vec::new();

    for (ai, a) in small.to_vec().iter().enumerate() {
        for (bi, b) in big.to_vec().iter().enumerate() {
            let distance = enum_dispatch!(a, b, a.EuclideanDistance(b));
            if distance <= max_distance {
                result.push(ProxMapRow {
                    big_index: bi,
                    small_index: ai,
                    distance,
                });
            }
        }
    }

    result.sort();
    result
}

pub(crate) fn slow_spatial_join(
    small: &SplitGeoSeq,
    big: &SplitGeoSeq,
    interaction: Interaction,
) -> Vec<SJoinRow> {
    let mut result = Vec::new();

    for (ai, a) in small.to_vec().iter().enumerate() {
        for (bi, b) in big.to_vec().iter().enumerate() {
            let include = match interaction {
                Interaction::Intersects => enum_dispatch!(a, b, a.Intersects(b)),
                Interaction::Contains => enum_dispatch!(a, b, a.Contains(b)),
                Interaction::Within => enum_dispatch!(a, b, b.Contains(a)),
            };
            if include {
                result.push(SJoinRow {
                    small_index: ai,
                    big_index: bi,
                });
            }
        }
    }

    result.sort();
    result
}
