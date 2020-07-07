from math import pi, cos, sin
from random import uniform

from shapely.geometry import LineString, GeometryCollection
import geopandas as gpd

longitudes = (-80, -40)
latitudes = (15, 45)


def rand_pt():
    lon = uniform(*longitudes)
    lat = uniform(*latitudes)
    return (lon, lat)

def rand_poly(max_distance_degrees = 0.5):
    start = rand_pt()
    distance = uniform(0, max_distance_degrees)
    angle_rad = uniform(0, 2*pi)
    end = (start[0] + cos(angle_rad) * distance,
           start[1] + sin(angle_rad) * distance)
    return LineString((start, end)).buffer(0.025)


def make(n, output):
    shapes = GeometryCollection([rand_poly() for _ in range(n)])

    if output.suffix == '.wkt':
        output.write_text(shapes.wkt)
    elif output.suffix == '.wkb':
        output.write_bytes(shapes.wkb)
    else:
        raise ValueError


from shapely.wkt import loads

def compare_sjoin():
    w = gpd.GeoDataFrame(geometry=list(loads(Path('polys1k.wkt').read_text())))
    gpd.sjoin(w, w, how='inner', op='intersects')
