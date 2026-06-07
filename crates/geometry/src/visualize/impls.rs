use geo::Coord;

use crate::{
    osm::Path,
    shard::Shard,
    visualize::{MapVisualizer, Style, Visualize},
};

impl Visualize for Shard {
    fn visualize(&self, visualizer: &mut MapVisualizer) {
        let line_style = Style::default().stroke("#e32d27").weight(3.0).opacity(1.0);

        for polygon in self.bounds.iter() {
            visualizer.add_line(polygon.exterior(), line_style.clone());
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

impl<T: Visualize> Visualize for Vec<T> {
    fn visualize(&self, visualizer: &mut MapVisualizer) {
        for item in self {
            item.visualize(visualizer);
        }
    }
}

pub struct Label {
    pub text: String,
    pub coord: Coord
}

impl Visualize for Label {
    fn visualize(&self, visualizer: &mut MapVisualizer) {
        visualizer.add_label(&self.coord, &self.text, Style::default().fill_color("white"));
    }
}