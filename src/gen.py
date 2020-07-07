from dataclasses import dataclass, replace
import typing as T

@dataclass
class Method:
    name: str
    type: str

Contains, Intersects, EuclideanDistance = Method('Contains', 'bool'), Method('Intersects', 'bool'), Method('EuclideanDistance', 'f64')

@dataclass
class GeoType:
    name: str
    dimensions: int

types = (GeoType('Point', 0), GeoType('Line', 1),
         GeoType('LineString', 1), GeoType('Polygon', 2),
         GeoType('Rect', 2), GeoType('Triangle', 2))

@dataclass
class MethodImpl:
    method: Method
    self_t: str
    other_t: str
    body: str

    def __str__(self):
        prefix = '_' if 'other' not in self.body else ''
        return (f'    fn {self.method.name}(&self,  {prefix}other: &{self.other_t}<f64>) -> {self.method.type} ' +
                '{\n        ' + self.body + '\n    }')


@dataclass
class Group:
    self_t: str
    other_t: str
    impls: T.List[MethodImpl]

    def __str__(self):
        header = f'impl Relates<{self.other_t}<f64>> for {self.self_t}<f64> ' + '{\n'
        return header + '\n'.join(map(str, self.impls)) + '\n}\n'

    @staticmethod
    def from_impls(impls: T.Sequence[MethodImpl]) -> T.List['Group']:
        groups = {}
        for impl in impls:
            key = (impl.self_t, impl.other_t)
            if key not in groups:
                groups[key] = Group(*key, [])
            groups[key].impls.append(impl)
        for g in groups.values():
            g.impls.sort(key=lambda mi: mi.method.name)
        return list(groups.values())


def allothers(seq: T.Sequence[MethodImpl],
              method: Method, body: str) -> T.Sequence[MethodImpl]:
    seq = list(seq)
    type_pairs = {(mi.self_t, mi.other_t) for mi in seq}
    return seq + [MethodImpl(method, a.name, b.name, body)
                  for a in types
                  for b in types
                  if (a.name, b.name) not in type_pairs]


def swapped(seq: T.Sequence[MethodImpl]) ->  T.Sequence[MethodImpl]:
    seq = list(seq)
    return seq + [
        replace(
            mi, self_t=mi.other_t, other_t=mi.self_t,
            body=f'other.{mi.method.name}(self)')
        for mi in seq
        if mi.self_t != mi.other_t]


contains = allothers(
    [MethodImpl(Contains, a.name, b.name, 'false')
     for a in types
     for b in types
     if a.dimensions < b.dimensions] +
    [MethodImpl(Contains, 'LineString', 'LineString', 'other.lines().all(|oline| self.lines().any(|sline| sline.Contains(&oline)))'),
     MethodImpl(Contains, 'Rect', 'Line', 'self.contains(&other.start_point()) && self.contains(&other.end_point())'),
     MethodImpl(Contains, 'Rect', 'LineString', 'other.points_iter().all(|pt| self.Contains(&pt))'),
     MethodImpl(Contains, 'Rect', 'Triangle', 'self.contains(&Point(other.0)) && self.contains(&Point(other.1)) && self.contains(&Point(other.2))'),
     MethodImpl(Contains, 'Rect', 'Polygon', 'other.exterior().points_iter().all(|pt| self.Contains(&pt))'),
     # FIX for bug in geo-types, https://github.com/georust/geo/issues/473, delete when they fix it:
     MethodImpl(Contains, 'Triangle', 'Point', 'if self.0 == self.1 && self.1 == self.2 {self.0 == other.0} else {self.contains(other)}'),
     # We're using Contains instead of contains only to work around the geo-types bug
     MethodImpl(Contains, 'Triangle', 'Line', 'self.Contains(&other.start_point()) && self.Contains(&other.end_point())'),
     MethodImpl(Contains, 'Triangle', 'LineString', 'other.lines().all(|line| self.Contains(&line))'),
     # We're using Contains instead of contains only to work around the geo-types bug
     MethodImpl(Contains, 'Triangle', 'Triangle', 'self.Contains(&Point(other.0)) && self.Contains(&Point(other.1)) && self.Contains(&Point(other.2))'),
     MethodImpl(Contains, 'Triangle', 'Polygon', 'other.exterior().points_iter().all(|pt| self.Contains(&pt))'),
     MethodImpl(Contains, 'Triangle', 'Rect', 'rect_lines(other).iter().all(|line| self.Contains(line))'),
     MethodImpl(Contains, 'Polygon', 'Rect', 'rect_lines(other).iter().all(|line| self.contains(line))'),
     MethodImpl(Contains, 'Polygon', 'Triangle', 'self.contains(&Point(other.0)) && self.contains(&Point(other.1)) && self.contains(&Point(other.2))'),
    ],
    Contains, 'self.contains(other)')


intersects = allothers(
    swapped(
        [MethodImpl(Intersects, 'Point', 'Point', 'self == other'), # FIXME: should be relative_eq!(0.0, line distance between self and other) to match geo-types
         MethodImpl(Intersects, 'Polygon', 'Point', 'self.contains(other)'),
         MethodImpl(Intersects, 'LineString', 'Point', 'self.contains(other)'),
         MethodImpl(Intersects, 'Rect', 'Point', 'self.Contains(other)'),
         MethodImpl(Intersects, 'Rect', 'Line',       'self.Contains(other) || rect_lines(self).iter().any(|sline| sline.intersects(other))'),
         MethodImpl(Intersects, 'Rect', 'LineString', 'self.Contains(other) || rect_lines(self).iter().any(|sline| other.lines().any(|oline| sline.intersects(&oline)))'),
         MethodImpl(Intersects, 'Rect', 'Triangle',   'self.Contains(other) || rect_lines(self).iter().any(|sline| other.to_lines().iter().any(|oline| sline.intersects(oline))) || other.Contains(self)'),
         MethodImpl(Intersects, 'Triangle', 'Point', 'self.Contains(other)'),
         MethodImpl(Intersects, 'Triangle', 'Line',       'self.Contains(other) || self.to_lines().iter().any(|sline| sline.intersects(other))'),
         MethodImpl(Intersects, 'Triangle', 'LineString', 'self.Contains(other) || self.to_lines().iter().any(|sline| other.lines().any(|oline| sline.intersects(&oline)))'),
         MethodImpl(Intersects, 'Triangle', 'Triangle',   'self.Contains(other) || self.to_lines().iter().any(|sline| other.to_lines().iter().any(|oline| sline.intersects(oline))) || other.Contains(self)'),
         MethodImpl(Intersects, 'Triangle', 'Polygon', 'self.Intersects(other.exterior()) || (other.exterior().Contains(self) || if other.interiors().is_empty() {false} else {other.interiors().iter().all(|hole| !hole.Contains(self))} )'),
        ]),
    Intersects, 'self.intersects(other)')


# https://github.com/georust/geo/issues/476 means that Rect/Polygon distances and probably Tri/Poly are busted
dists = allothers(
    swapped(
    [MethodImpl(EuclideanDistance, 'Polygon', 'Line', 'if self.intersects(other) {0.0} else {self.exterior().lines().map(|sline| sline.euclidean_distance(other)).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()}'), # this is only needed until https://github.com/georust/geo/issues/476 gets fixed
     MethodImpl(EuclideanDistance, 'Rect',       'Point', 'if self.Intersects(other) {0.0} else {rect_lines(self).iter().map(|sline| sline.euclidean_distance(other)).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()}'),
     MethodImpl(EuclideanDistance, 'Rect',       'Line', 'if self.Intersects(other) {0.0} else {rect_lines(self).iter().map(|sline| sline.euclidean_distance(other)).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()}'),
     MethodImpl(EuclideanDistance, 'Rect',       'LineString', 'if self.Intersects(other) {0.0} else {rect_lines(self).iter().map(|sline| sline.euclidean_distance(other)).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()}'),
     MethodImpl(EuclideanDistance, 'Rect',       'Triangle', 'if self.Intersects(other) {0.0} else {rect_lines(self).iter().map(|sline| sline.EuclideanDistance(other)).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()}'),
     MethodImpl(EuclideanDistance, 'Rect',       'Rect', 'if self.Intersects(other) {0.0} else {rect_lines(other).iter().map(|oline| oline.EuclideanDistance(self)).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()}'),
     MethodImpl(EuclideanDistance, 'Rect',       'Polygon', 'if self.Intersects(other.exterior()) {0.0} else {rect_lines(self).iter().map(|sline| sline.EuclideanDistance(other)).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()}'), # we can switch back to euclidean_distance after 476 gets fixed
     MethodImpl(EuclideanDistance, 'Triangle',   'Point', 'if self.Intersects(other) {0.0} else {self.to_lines().iter().map(|sline| sline.euclidean_distance(other)).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()}'),
     MethodImpl(EuclideanDistance, 'Triangle',   'Line', 'if self.Intersects(other) {0.0} else {self.to_lines().iter().map(|sline| sline.euclidean_distance(other)).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()}'),
     MethodImpl(EuclideanDistance, 'Triangle',   'LineString', 'if self.Intersects(other) {0.0} else {self.to_lines().iter().map(|sline| other.lines().map(|oline| oline.euclidean_distance(sline)).min_by(|a, b| a.partial_cmp(b).unwrap())).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap().unwrap()}'),
     MethodImpl(EuclideanDistance, 'Triangle',   'Triangle', 'if self.Intersects(other) {0.0} else {other.to_lines().iter().map(|oline| self.to_lines().iter().map(|sline| sline.euclidean_distance(oline)).min_by(|a, b| a.partial_cmp(b).unwrap())).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap().unwrap()}'),
     MethodImpl(EuclideanDistance, 'Triangle',   'Polygon', 'if self.Intersects(other.exterior()) {0.0} else {self.to_lines().iter().map(|sline| sline.EuclideanDistance(other)).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()}'), # we can switch back to euclidean_distance after 476 gets fixed
     ]), EuclideanDistance, 'self.euclidean_distance(other)')


for group in Group.from_impls(contains + intersects + dists):
    print(group)
