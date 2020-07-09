use std::convert::TryInto;

use smallvec::SmallVec;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use crate::rtrees::Envelope;
use crate::{
    Config, Error, Interaction, ProxMapGeoRow, ProxMapRow, SJoinGeoRow, SJoinRow, SpatialIndex,
    SplitGeoSeq,
};
#[cfg(feature = "parallel")]
use crate::{Par, ParSpatialIndex};

use crate::relates::Relates;

macro_rules! chain {
    ($thing:expr) => ($thing);
    ($head:expr, $($tail:expr),+) => ($head.chain(chain!($($tail),+)));

}

macro_rules! join_outer {
    ($big:expr;
     $geo_big:ident, $ext_index_big:ident, $env:ident;
     $expr_copyable:expr, $expr_noncopyable:expr) => {
        chain!(
            $big.geos
                .points
                .into_iter()
                .zip($big.indexes.points.into_iter())
                .flat_map(move |($geo_big, $ext_index_big)| {
                    let $env = $geo_big.to_env();
                    $expr_copyable
                }),
            $big.geos
                .lines
                .into_iter()
                .zip($big.indexes.lines.into_iter())
                .flat_map(move |($geo_big, $ext_index_big)| {
                    let $env = $geo_big.to_env();
                    $expr_copyable
                }),
            $big.geos
                .rects
                .into_iter()
                .zip($big.indexes.rects.into_iter())
                .flat_map(move |($geo_big, $ext_index_big)| {
                    let $env = $geo_big.to_env();
                    $expr_copyable
                }),
            $big.geos
                .tris
                .into_iter()
                .zip($big.indexes.tris.into_iter())
                .flat_map(move |($geo_big, $ext_index_big)| {
                    let $env = $geo_big.to_env();
                    $expr_copyable
                }),
            $big.geos
                .polys
                .into_iter()
                .zip($big.indexes.polys.into_iter())
                .flat_map(move |($geo_big, $ext_index_big)| {
                    let $env = $geo_big.to_env();
                    let $geo_big = &$geo_big;
                    $expr_noncopyable.into_iter()
                }),
            $big.geos
                .line_strings
                .into_iter()
                .zip($big.indexes.line_strings.into_iter())
                .flat_map(move |($geo_big, $ext_index_big)| {
                    let $env = $geo_big.to_env();
                    let $geo_big = &$geo_big;
                    $expr_noncopyable.into_iter()
                })
        )
    };
}

#[cfg(feature = "parallel")]
macro_rules! par_join_outer {
    ($big:expr;
     $geo_big:ident, $ext_index_big:ident, $env:ident;
     $expr_copyable:expr, $expr_noncopyable:expr) => {{
        chain!(
            $big.geos
                .points
                .into_par_iter()
                .zip($big.indexes.points.into_par_iter())
                .flat_map(move |($geo_big, $ext_index_big)| {
                    let $env = $geo_big.to_env();
                    ($expr_copyable).par_bridge()
                }),
            $big.geos
                .lines
                .into_par_iter()
                .zip($big.indexes.lines.into_par_iter())
                .flat_map(move |($geo_big, $ext_index_big)| {
                    let $env = $geo_big.to_env();
                    ($expr_copyable).par_bridge()
                }),
            $big.geos
                .rects
                .into_par_iter()
                .zip($big.indexes.rects.into_par_iter())
                .flat_map(move |($geo_big, $ext_index_big)| {
                    let $env = $geo_big.to_env();
                    ($expr_copyable).par_bridge()
                }),
            $big.geos
                .tris
                .into_par_iter()
                .zip($big.indexes.tris.into_par_iter())
                .flat_map(move |($geo_big, $ext_index_big)| {
                    let $env = $geo_big.to_env();
                    ($expr_copyable).par_bridge()
                }),
            $big.geos
                .polys
                .into_par_iter()
                .zip($big.indexes.polys.into_par_iter())
                .flat_map(move |($geo_big, $ext_index_big)| {
                    let $env = $geo_big.to_env();
                    let $geo_big = &$geo_big;
                    $expr_noncopyable.into_iter().par_bridge()
                }),
            $big.geos
                .line_strings
                .into_par_iter()
                .zip($big.indexes.line_strings.into_par_iter())
                .flat_map(move |($geo_big, $ext_index_big)| {
                    let $env = $geo_big.to_env();
                    let $geo_big = &$geo_big;
                    $expr_noncopyable.into_iter().par_bridge()
                })
        )
    }};
}

macro_rules! join_inner_copyable {
    ($pm:expr;
     $geo_big:ident, $ext_index_big:ident, $env:ident,
     $geo_small:ident, $ext_index_small:ident;
     $expr:expr) => {{
        chain!(
            $pm.point_tree
                .locate_in_envelope_intersecting(&$env)
                .map(|fake| fake.id)
                .filter_map({
                    let $geo_big = $geo_big.clone();
                    move |index_small| {
                        let $geo_small = &$pm.small.geos.points[index_small];
                        let $ext_index_small = $pm.small.indexes.points.get(index_small);
                        let $geo_big = &$geo_big;
                        $expr
                    }
                }),
            $pm.line_tree
                .locate_in_envelope_intersecting(&$env)
                .map(|fake| fake.id)
                .filter_map({
                    let $geo_big = $geo_big.clone();
                    move |index_small| {
                        let $geo_small = &$pm.small.geos.lines[index_small];
                        let $ext_index_small = $pm.small.indexes.lines.get(index_small);
                        let $geo_big = &$geo_big;
                        $expr
                    }
                }),
            $pm.poly_tree
                .locate_in_envelope_intersecting(&$env)
                .map(|fake| fake.id)
                .filter_map({
                    move |index_small| {
                        let $geo_small = &$pm.small.geos.polys[index_small];
                        let $ext_index_small = $pm.small.indexes.polys.get(index_small);
                        let $geo_big = &$geo_big;
                        $expr
                    }
                }),
            $pm.ls_tree
                .locate_in_envelope_intersecting(&$env)
                .map(|fake| fake.id)
                .filter_map({
                    move |index_small| {
                        let $geo_small = &$pm.small.geos.line_strings[index_small];
                        let $ext_index_small = $pm.small.indexes.line_strings.get(index_small);
                        let $geo_big = &$geo_big;
                        $expr
                    }
                }),
            $pm.rect_tree
                .locate_in_envelope_intersecting(&$env)
                .map(|fake| fake.id)
                .filter_map({
                    let $geo_big = $geo_big.clone();
                    move |index_small| {
                        let $geo_small = &$pm.small.geos.rects[index_small];
                        let $ext_index_small = $pm.small.indexes.rects.get(index_small);
                        let $geo_big = &$geo_big;
                        $expr
                    }
                }),
            $pm.tri_tree
                .locate_in_envelope_intersecting(&$env)
                .map(|fake| fake.id)
                .filter_map({
                    let $geo_big = $geo_big.clone();
                    move |index_small| {
                        let $geo_small = &$pm.small.geos.tris[index_small];
                        let $ext_index_small = $pm.small.indexes.tris.get(index_small);
                        let $geo_big = &$geo_big;
                        $expr
                    }
                })
        )
    }};
}

macro_rules! join_inner_noncopyable {
    ($pm:expr; $expr_type:ty;
     $geo_big:ident, $ext_index_big:ident, $env:ident,
     $geo_small:ident, $ext_index_small:ident;
     $expr:expr) => {{
        let mut result = SmallVec::<[$expr_type; 10]>::new();
        result.extend(
            $pm.point_tree
                .locate_in_envelope_intersecting(&$env)
                .map(|fake| fake.id)
                .filter_map({
                    move |index_small| {
                        let $geo_small = &$pm.small.geos.points[index_small];
                        let $ext_index_small = $pm.small.indexes.points.get(index_small);
                        $expr
                    }
                }),
        );
        result.extend(
            $pm.line_tree
                .locate_in_envelope_intersecting(&$env)
                .map(|fake| fake.id)
                .filter_map({
                    move |index_small| {
                        let $geo_small = &$pm.small.geos.lines[index_small];
                        let $ext_index_small = $pm.small.indexes.lines.get(index_small);
                        $expr
                    }
                }),
        );
        result.extend(
            $pm.poly_tree
                .locate_in_envelope_intersecting(&$env)
                .map(|fake| fake.id)
                .filter_map({
                    move |index_small| {
                        let $geo_small = &$pm.small.geos.polys[index_small];
                        let $ext_index_small = $pm.small.indexes.polys.get(index_small);
                        $expr
                    }
                }),
        );
        result.extend(
            $pm.ls_tree
                .locate_in_envelope_intersecting(&$env)
                .map(|fake| fake.id)
                .filter_map({
                    move |index_small| {
                        let $geo_small = &$pm.small.geos.line_strings[index_small];
                        let $ext_index_small = $pm.small.indexes.line_strings.get(index_small);
                        $expr
                    }
                }),
        );
        result.extend(
            $pm.rect_tree
                .locate_in_envelope_intersecting(&$env)
                .map(|fake| fake.id)
                .filter_map({
                    move |index_small| {
                        let $geo_small = &$pm.small.geos.rects[index_small];
                        let $ext_index_small = $pm.small.indexes.rects.get(index_small);
                        $expr
                    }
                }),
        );
        result.extend(
            $pm.tri_tree
                .locate_in_envelope_intersecting(&$env)
                .map(|fake| fake.id)
                .filter_map({
                    move |index_small| {
                        let $geo_small = &$pm.small.geos.tris[index_small];
                        let $ext_index_small = $pm.small.indexes.tris.get(index_small);
                        $expr
                    }
                }),
        );
        result
    }};
}

macro_rules! join {
    ($pm:expr,
     $expr_type:ty,
     $big:expr;

     $geo_big:ident, $ext_index_big:ident, $env:ident,
     $geo_small:ident, $ext_index_small:ident;

     $expr:expr) => {
        join_outer!(
            $big;
            $geo_big,
            $ext_index_big,
            $env;
            join_inner_copyable!(
                $pm;
                $geo_big,
                $ext_index_big,
                $env,
                $geo_small,
                $ext_index_small;
		$expr
            ),
	    join_inner_noncopyable!(
                $pm; $expr_type;
                $geo_big,
                $ext_index_big,
                $env,
                $geo_small,
                $ext_index_small;
		$expr
            )

        )
    }
}

#[cfg(feature = "parallel")]
macro_rules! par_join {
    ($pm:expr,
     $expr_type:ty,
     $big:expr;

     $geo_big:ident, $ext_index_big:ident, $env:ident,
     $geo_small:ident, $ext_index_small:ident;

     $expr:expr) => {
        par_join_outer!(
            $big;
            $geo_big,
            $ext_index_big,
            $env;
            join_inner_copyable!(
                $pm;
                $geo_big,
                $ext_index_big,
                $env,
                $geo_small,
                $ext_index_small;
		$expr
            ),
	    join_inner_noncopyable!(
                $pm; $expr_type;
                $geo_big,
                $ext_index_big,
                $env,
                $geo_small,
                $ext_index_small;
		$expr
            )

        )
    }
}

pub(crate) fn sgs_try_into<T, U>(thing: T) -> Result<SplitGeoSeq, Error>
where
    T: TryInto<SplitGeoSeq, Error = U>,
    U: std::any::Any,
{
    let thing: Result<SplitGeoSeq, _> = thing.try_into();
    // FIXME: maybe map_error
    match thing {
        Ok(thing) => {
            // conversion into SplitGeoSeq worked!
            Ok(thing)
        }
        Err(e) => {
            let any_e = &e as &dyn std::any::Any;
            Err(any_e.downcast_ref::<Error>().expect("impossible").clone())
        }
    }
}

impl SpatialIndex {
    pub fn new<T, U>(small: T, config: Config) -> Result<Self, Error>
    where
        T: TryInto<SplitGeoSeq, Error = U>,
        U: std::any::Any,
    {
        let max_distance = config.max_distance;
        let small = sgs_try_into(small)?;

        let [point_tree, line_tree, poly_tree, ls_tree, rect_tree, tri_tree] =
            small.to_rtrees(max_distance);
        Ok(SpatialIndex {
            small,
            point_tree,
            line_tree,
            poly_tree,
            ls_tree,
            rect_tree,
            tri_tree,
            config,
        })
    }

    pub fn proximity_map<'a, T: 'a, U>(
        &'a self,
        big: T,
    ) -> Result<impl Iterator<Item = ProxMapRow> + 'a, Error>
    where
        T: TryInto<SplitGeoSeq, Error = U>,
        U: std::any::Any + std::fmt::Debug,
    {
        let big = sgs_try_into(big)?;
        Ok(join!(self, ProxMapRow, big;
                  geo_big, ext_index_big, env,
                  geo_small, ext_index_small;
                  {
              let distance = geo_big.EuclideanDistance(geo_small);
              assert!(distance.is_finite());

              if distance <= self.config.max_distance {
                  Some(ProxMapRow {big_index: ext_index_big,
                   small_index: ext_index_small,
                   distance})
              } else {
                  None
              }
                  }
        ))
    }

    pub fn proximity_map_with_geos<'a, T: 'a, U>(
        &'a self,
        big: T,
    ) -> Result<impl Iterator<Item = ProxMapGeoRow> + 'a, Error>
    where
        T: TryInto<SplitGeoSeq, Error = U>,
        U: std::any::Any + std::fmt::Debug,
    {
        let big = sgs_try_into(big)?;
        Ok(join!(self, ProxMapGeoRow, big;
                geo_big, ext_index_big, env,
                geo_small, ext_index_small;
                {
            let distance = geo_big.EuclideanDistance(geo_small);
            assert!(distance.is_finite());

            if distance <= self.config.max_distance {
        Some(ProxMapGeoRow {big_index: ext_index_big,
                small_index: ext_index_small,
              big: geo_big.clone().into(),
                small: geo_small.clone().into(), distance})
            } else {
                None
            }
                }
          ))
    }

    pub fn spatial_join<'a, T, U>(
        &'a self,
        big: T,
        interaction: Interaction,
    ) -> Result<impl Iterator<Item = SJoinRow> + 'a, Error>
    where
        T: TryInto<SplitGeoSeq, Error = U>,
        U: std::any::Any + std::fmt::Debug,
    {
        let big = sgs_try_into(big)?;
        // This is a weird structure designed to solve an odd
        // problem. For performance, I want to have monomorphized code
        // for each `Interaction` branch; in other words, I don't want
        // to do a `match interaction` in the innermost loop. But
        // trying to put a `match interaction` as the main body of
        // this method fails since the different arms have different
        // types that can't be unified! Hence this approach: we always
        // run all three arms and chain them together, but two of the
        // three arms get empty inputs. That way we satisfy the type
        // system since we always emit the same type.
        let (big_intersects, big_contains, big_within) = match interaction {
            Interaction::Intersects => (big, SplitGeoSeq::default(), SplitGeoSeq::default()),
            Interaction::Contains => (SplitGeoSeq::default(), big, SplitGeoSeq::default()),
            Interaction::Within => (SplitGeoSeq::default(), SplitGeoSeq::default(), big),
        };
        Ok(chain!(
            // These calls are identical except for the big_ variable
            // and the geo_big.Interaction call.
            join!(self, SJoinRow, big_intersects;
                    geo_big, ext_index_big, env,
                    geo_small, ext_index_small;

            if geo_small.Intersects(geo_big)  {
                Some(SJoinRow {big_index: ext_index_big, small_index: ext_index_small })
                    } else {
                        None
                    }
              ),
            join!(self, SJoinRow, big_contains;
                    geo_big, ext_index_big, env,
                    geo_small, ext_index_small;
            if geo_small.Contains(geo_big) {
                Some(SJoinRow {big_index: ext_index_big, small_index: ext_index_small})
                    } else {
                        None
                    }
              ),
            join!(self, SJoinRow, big_within;
                    geo_big, ext_index_big, env,
                    geo_small, ext_index_small;

            if (geo_big).Contains(geo_small) {
                Some(SJoinRow {big_index: ext_index_big, small_index: ext_index_small})
                    } else {
                        None
                    }
              )
        ))
    }

    pub fn spatial_join_with_geos<'a, T, U>(
        &'a self,
        big: T,
        interaction: Interaction,
    ) -> Result<impl Iterator<Item = SJoinGeoRow> + 'a, Error>
    where
        T: TryInto<SplitGeoSeq, Error = U>,
        U: std::any::Any + std::fmt::Debug,
    {
        let big = sgs_try_into(big)?;

        // This is a weird structure designed to solve an odd
        // problem. For performance, I want to have monomorphized code
        // for each `Interaction` branch; in other words, I don't want
        // to do a `match interaction` in the innermost loop. But
        // trying to put a `match interaction` as the main body of
        // this method fails since the different arms have different
        // types that can't be unified! Hence this approach: we always
        // run all three arms and chain them together, but two of the
        // three arms get empty inputs. That way we satisfy the type
        // system since we always emit the same type.
        let (big_intersects, big_contains, big_within) = match interaction {
            Interaction::Intersects => (big, SplitGeoSeq::default(), SplitGeoSeq::default()),
            Interaction::Contains => (SplitGeoSeq::default(), big, SplitGeoSeq::default()),
            Interaction::Within => (SplitGeoSeq::default(), SplitGeoSeq::default(), big),
        };
        Ok(chain!(
            // These calls are identical except for the big_ variable
            // and the geo_big.Interaction call.
            join!(self, SJoinGeoRow, big_intersects;
                        geo_big, ext_index_big, env,
                        geo_small, ext_index_small;

                if geo_small.Intersects(geo_big)  {
            Some(SJoinGeoRow{big_index: ext_index_big, small_index: ext_index_small,
                     big: geo_big.clone().into(), small: geo_small.clone().into()})
                        } else {
                            None
                        }
                  ),
            join!(self, SJoinGeoRow, big_contains;
                        geo_big, ext_index_big, env,
                        geo_small, ext_index_small;
                if geo_small.Contains(geo_big) {
            Some(SJoinGeoRow{big_index: ext_index_big, small_index: ext_index_small,
                     big: geo_big.clone().into(), small: geo_small.clone().into()})
                        } else {
                            None
                        }
                  ),
            join!(self, SJoinGeoRow, big_within;
                        geo_big, ext_index_big, env,
                        geo_small, ext_index_small;

                if geo_big.Contains(geo_small) {
            Some(SJoinGeoRow{big_index: ext_index_big, small_index: ext_index_small,
                     big: geo_big.clone().into(), small: geo_small.clone().into()})
                        } else {
                            None
                        }
                  )
        ))
    }
}

#[cfg(feature = "parallel")]
pub(crate) fn par_sgs_try_into<T, U>(thing: T) -> Result<SplitGeoSeq, Error>
where
    T: TryInto<Par<SplitGeoSeq>, Error = U>,
    U: std::any::Any,
{
    let thing: Result<Par<SplitGeoSeq>, _> = thing.try_into();
    // FIXME: maybe map_error
    match thing {
        Ok(thing) => {
            // conversion into SplitGeoSeq worked!
            Ok(thing.0)
        }
        Err(e) => {
            let any_e = &e as &dyn std::any::Any;
            Err(any_e.downcast_ref::<Error>().expect("impossible").clone())
        }
    }
}

// We have to handle parallel with separate methods because parallel
// and serial iterator have different types.

#[cfg(feature = "parallel")]
impl ParSpatialIndex {
    pub fn new<T, U>(small: T, config: Config) -> Result<Self, Error>
    where
        T: TryInto<Par<SplitGeoSeq>, Error = U>,
        U: std::any::Any,
    {
        let max_distance = config.max_distance;
        let small = par_sgs_try_into(small)?;

        let [point_tree, line_tree, poly_tree, ls_tree, rect_tree, tri_tree] =
            small.to_rtrees(max_distance);
        Ok(ParSpatialIndex(SpatialIndex {
            small,
            point_tree,
            line_tree,
            poly_tree,
            ls_tree,
            rect_tree,
            tri_tree,
            config,
        }))
    }

    pub fn proximity_map<'a, T: 'a, U>(
        &'a self,
        big: T,
    ) -> Result<impl ParallelIterator<Item = ProxMapRow> + 'a, Error>
    where
        T: TryInto<Par<SplitGeoSeq>, Error = U>,
        U: std::any::Any + std::fmt::Debug,
    {
        let big = par_sgs_try_into(big)?;

        Ok(par_join!(self.0, ProxMapRow, big;
                  geo_big, ext_index_big, env,
                  geo_small, ext_index_small;
                  {
              let distance = geo_big.EuclideanDistance(geo_small);
              assert!(distance.is_finite());

              if distance <= self.0.config.max_distance {
                  Some(ProxMapRow {big_index: ext_index_big,
                   small_index: ext_index_small,
                   distance})
              } else {
                  None
              }
                  }
        ))
    }

    pub fn proximity_map_with_geos<'a, T: 'a, U>(
        &'a self,
        big: T,
    ) -> Result<impl ParallelIterator<Item = ProxMapGeoRow> + 'a, Error>
    where
        T: TryInto<Par<SplitGeoSeq>, Error = U>,
        U: std::any::Any + std::fmt::Debug,
    {
        let big = par_sgs_try_into(big)?;
        Ok(par_join!(self.0, ProxMapGeoRow, big;
                geo_big, ext_index_big, env,
                geo_small, ext_index_small;
                {
            let distance = geo_big.EuclideanDistance(geo_small);
            assert!(distance.is_finite());

            if distance <= self.0.config.max_distance {
        Some(ProxMapGeoRow {big_index: ext_index_big,
                small_index: ext_index_small,
              big: geo_big.clone().into(),
                small: geo_small.clone().into(), distance})
            } else {
                None
            }
                }
          ))
    }

    pub fn spatial_join<'a, T, U>(
        &'a self,
        big: T,
        interaction: Interaction,
    ) -> Result<impl ParallelIterator<Item = SJoinRow> + 'a, Error>
    where
        T: TryInto<Par<SplitGeoSeq>, Error = U>,
        U: std::any::Any + std::fmt::Debug,
    {
        let big = par_sgs_try_into(big)?;
        // This is a weird structure designed to solve an odd
        // problem. For performance, I want to have monomorphized code
        // for each `Interaction` branch; in other words, I don't want
        // to do a `match interaction` in the innermost loop. But
        // trying to put a `match interaction` as the main body of
        // this method fails since the different arms have different
        // types that can't be unified! Hence this approach: we always
        // run all three arms and chain them together, but two of the
        // three arms get empty inputs. That way we satisfy the type
        // system since we always emit the same type.
        let (big_intersects, big_contains, big_within) = match interaction {
            Interaction::Intersects => (big, SplitGeoSeq::default(), SplitGeoSeq::default()),
            Interaction::Contains => (SplitGeoSeq::default(), big, SplitGeoSeq::default()),
            Interaction::Within => (SplitGeoSeq::default(), SplitGeoSeq::default(), big),
        };
        Ok(chain!(
            // These calls are identical except for the big_ variable
            // and the geo_big.Interaction call.
            par_join!(self.0, SJoinRow, big_intersects;
                    geo_big, ext_index_big, env,
                    geo_small, ext_index_small;

            if geo_small.Intersects(geo_big)  {
                Some(SJoinRow {big_index: ext_index_big, small_index: ext_index_small })
                    } else {
                        None
                    }
              ),
            par_join!(self.0, SJoinRow, big_contains;
                    geo_big, ext_index_big, env,
                    geo_small, ext_index_small;
            if geo_small.Contains(geo_big) {
                Some(SJoinRow {big_index: ext_index_big, small_index: ext_index_small})
                    } else {
                        None
                    }
              ),
            par_join!(self.0, SJoinRow, big_within;
                    geo_big, ext_index_big, env,
                    geo_small, ext_index_small;

            if (geo_big).Contains(geo_small) {
                Some(SJoinRow {big_index: ext_index_big, small_index: ext_index_small})
                    } else {
                        None
                    }
              )
        ))
    }

    pub fn spatial_join_with_geos<'a, T, U>(
        &'a self,
        big: T,
        interaction: Interaction,
    ) -> Result<impl ParallelIterator<Item = SJoinGeoRow> + 'a, Error>
    where
        T: TryInto<Par<SplitGeoSeq>, Error = U>,
        U: std::any::Any + std::fmt::Debug,
    {
        let big = par_sgs_try_into(big)?;

        // This is a weird structure designed to solve an odd
        // problem. For performance, I want to have monomorphized code
        // for each `Interaction` branch; in other words, I don't want
        // to do a `match interaction` in the innermost loop. But
        // trying to put a `match interaction` as the main body of
        // this method fails since the different arms have different
        // types that can't be unified! Hence this approach: we always
        // run all three arms and chain them together, but two of the
        // three arms get empty inputs. That way we satisfy the type
        // system since we always emit the same type.
        let (big_intersects, big_contains, big_within) = match interaction {
            Interaction::Intersects => (big, SplitGeoSeq::default(), SplitGeoSeq::default()),
            Interaction::Contains => (SplitGeoSeq::default(), big, SplitGeoSeq::default()),
            Interaction::Within => (SplitGeoSeq::default(), SplitGeoSeq::default(), big),
        };
        Ok(chain!(
            // These calls are identical except for the big_ variable
            // and the geo_big.Interaction call.
            par_join!(self.0, SJoinGeoRow, big_intersects;
                        geo_big, ext_index_big, env,
                        geo_small, ext_index_small;

                if geo_small.Intersects(geo_big)  {
            Some(SJoinGeoRow{big_index: ext_index_big, small_index: ext_index_small,
                     big: geo_big.clone().into(), small: geo_small.clone().into()})
                        } else {
                            None
                        }
                  ),
            par_join!(self.0, SJoinGeoRow, big_contains;
                        geo_big, ext_index_big, env,
                        geo_small, ext_index_small;
                if geo_small.Contains(geo_big) {
            Some(SJoinGeoRow{big_index: ext_index_big, small_index: ext_index_small,
                     big: geo_big.clone().into(), small: geo_small.clone().into()})
                        } else {
                            None
                        }
                  ),
            par_join!(self.0, SJoinGeoRow, big_within;
                        geo_big, ext_index_big, env,
                        geo_small, ext_index_small;

                if geo_big.Contains(geo_small) {
            Some(SJoinGeoRow{big_index: ext_index_big, small_index: ext_index_small,
                     big: geo_big.clone().into(), small: geo_small.clone().into()})
                        } else {
                            None
                        }
                  )
        ))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn chain_macro() {
        let x: Vec<i32> = chain!(3..7, 9..12, 2..4).collect();
        assert_eq!(x, vec![3, 4, 5, 6, 9, 10, 11, 2, 3]);

        let x: Vec<i32> = chain!(3..7).collect();
        assert_eq!(x, vec![3, 4, 5, 6])
    }
}
