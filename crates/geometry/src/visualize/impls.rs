use geo::{BallTreePoint, Centroid, Coord};
use geo_traits::to_geo::{ToGeoGeometry, ToGeoPolygon};

use crate::{
    osm::Path,
    shard::{Shard, ShardCluster},
    visualize::{MapVisualizer, Style, Visualize},
};

impl Visualize for Shard {
    fn visualize(&self, visualizer: &mut MapVisualizer) {
        visualizer.add_label(
            &self
                .cell
                .boundary()
                .to_geometry()
                .centroid()
                .unwrap()
                .coord(),
            &self.cell.to_string(),
            Style::default().fill_color("white").opacity(0.5),
        );

        let line_style = Style::default().stroke("#e32d27").weight(3.0).opacity(1.0);
        visualizer.add_line(
            self.cell.boundary().to_polygon().exterior(),
            line_style.clone(),
        );

        for road in &self.paths {
            visualizer.draw(road);
        }
    }
}

impl Visualize for Path {
    fn visualize(&self, visualizer: &mut MapVisualizer) {
        visualizer.add_line(
            &self.geometry,
            Style::default().stroke("#e32d27").weight(2.0).opacity(0.6),
        );

        for segment in &self.segments {
            visualizer.add_point(segment, Style::default().fill_color("#1f7bf2").weight(1.0));
        }
    }
}

impl Visualize for ShardCluster {
    fn visualize(&self, visualizer: &mut MapVisualizer) {
        self.0.visualize(visualizer);
    }
}

impl<T: Visualize> Visualize for Vec<T> {
    fn visualize(&self, visualizer: &mut MapVisualizer) {
        for item in self {
            item.visualize(visualizer);
        }
    }
}

pub struct Label {
    pub text: String,
    pub coord: Coord,
}

impl Visualize for Label {
    fn visualize(&self, visualizer: &mut MapVisualizer) {
        visualizer.add_label(
            &self.coord,
            &self.text,
            Style::default().fill_color("white"),
        );
    }
}
