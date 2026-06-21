use geo::{Coord, Point};
use geo::{Distance, Haversine};
use rstar::{AABB, PointDistance, RTree, RTreeObject};

#[derive(Clone, Debug)]
struct SpatialNode<T> {
    coord: Coord,
    payload: T,
}

impl<T> RTreeObject for SpatialNode<T> {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.coord.x, self.coord.y])
    }
}

impl<T> PointDistance for SpatialNode<T> {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let dx = self.coord.x - point[0];
        let dy = self.coord.y - point[1];
        dx * dx + dy * dy
    }
}

pub struct SpatialTree<T> {
    inner: RTree<SpatialNode<T>>,
}

impl<T> SpatialTree<T> {
    pub fn new() -> Self {
        Self {
            inner: RTree::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.inner.size()
    }

    pub fn from_bulk(items: impl IntoIterator<Item = (Coord, T)>) -> Self {
        let nodes: Vec<SpatialNode<T>> = items
            .into_iter()
            .map(|(coord, payload)| SpatialNode { coord, payload })
            .collect();

        Self {
            inner: RTree::bulk_load(nodes),
        }
    }

    pub fn insert(&mut self, coord: Coord, payload: T) {
        self.inner.insert(SpatialNode { coord, payload });
    }

    /// Finds the FIRST item within the threshold radius, in meters
    pub fn get_at(&self, coord: Coord, threshold_radius_meters: f64) -> Option<&T> {
        let center_pt = Point::from(coord);

        self.get_candidates(coord, threshold_radius_meters)
            .find(|candidate| {
                let candidate_pt = Point::from(candidate.coord);
                Haversine.distance(center_pt, candidate_pt) <= threshold_radius_meters
            })
            .map(|node| &node.payload)
    }

    pub fn get_at_filter(
        &self,
        coord: Coord,
        threshold_radius_meters: f64,
        filter: impl Fn(&T) -> bool,
    ) -> Option<&T> {
        let center_pt = Point::from(coord);

        self.get_candidates(coord, threshold_radius_meters)
            .filter(|item| filter(&item.payload))
            .find(|candidate| {
                let candidate_pt = Point::from(candidate.coord);
                Haversine.distance(center_pt, candidate_pt) <= threshold_radius_meters
            })
            .map(|node| &node.payload)
    }

    pub fn get_intersecting<'a>(
        &self,
        points: &'a [Coord],
        radius: f64,
        reverse: bool,
    ) -> impl Iterator<Item = &'a Coord> {
        points.iter().filter(move |p| {
            let exists = self.get_at(**p, radius).is_some();
            reverse == exists
        })
    }

    fn get_candidates(
        &self,
        coord: Coord,
        threshold_radius_meters: f64,
    ) -> impl Iterator<Item = &SpatialNode<T>> {
        let lat_buffer = threshold_radius_meters / 111_111.0;
        let lon_buffer = lat_buffer / coord.y.to_radians().cos().abs();

        let search_envelope = AABB::from_corners(
            [coord.x - lon_buffer, coord.y - lat_buffer],
            [coord.x + lon_buffer, coord.y + lat_buffer],
        );

        self.inner.locate_in_envelope_intersecting(search_envelope)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner.iter().map(|ind| &ind.payload)
    }
}
