cases = 'Line Point Polygon LineString Rect Triangle'.split()
print("macro_rules! enum_dispatch { ($a:ident, $b:ident, $expr:expr) => { match ($a, $b) {")
for a in cases:
    for b in cases:
        print(f'(Geometry::{a}($a), Geometry::{b}($b)) => $expr,')
print('_ => panic!("match failure in enum_dispatch!")}}}')
