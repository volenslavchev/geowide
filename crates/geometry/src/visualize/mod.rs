mod impls;

use geo::{Coord, LineString, Point as GeoPoint, Polygon as GeoPolygon};
use geojson::{Feature, FeatureCollection, Geometry};
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub use impls::*;

pub trait Visualize {
    fn visualize(&self, visualizer: &mut MapVisualizer);
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Style {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_opacity: Option<f64>,
}

impl Style {
    pub fn stroke(mut self, stroke: &str) -> Self {
        self.stroke = Some(stroke.to_string());
        self
    }

    pub fn weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }

    pub fn opacity(mut self, opacity: f64) -> Self {
        self.opacity = Some(opacity);
        self
    }

    pub fn radius(mut self, radius: f64) -> Self {
        self.radius = Some(radius);
        self
    }

    pub fn fill_color(mut self, color: &str) -> Self {
        self.fill_color = Some(color.to_string());
        self
    }

    pub fn fill_opacity(mut self, opacity: f64) -> Self {
        self.fill_opacity = Some(opacity);
        self
    }
}

#[derive(Default)]
pub struct MapVisualizer {
    features: Vec<Feature>,
}

impl MapVisualizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn draw<Obj: Visualize>(&mut self, obj: &Obj) -> &mut Self {
        obj.visualize(self);
        self
    }

    pub fn add_line(&mut self, line: &LineString, style: Style) -> &mut Self {
        let geom = Geometry::from(line);
        self.push_feature(geom, style, "line");
        self
    }

    pub fn add_point(&mut self, coord: &Coord, style: Style) -> &mut Self {
        let point = GeoPoint::from(*coord);
        let geom = Geometry::from(&point);
        self.push_feature(geom, style, "circle_point");
        self
    }

    pub fn add_coords(&mut self, points: &[Coord], style: Style) -> &mut Self {
        for point in points {
            self.add_point(point, style.clone());
        }
        self
    }

    pub fn add_multipoint(&mut self, coords: &[Coord], style: Style) -> &mut Self {
        let points: Vec<GeoPoint> = coords.iter().map(|&c| GeoPoint::from(c)).collect();
        let multi_point = geo::MultiPoint::new(points);
        let geom = Geometry::from(&multi_point);
        self.push_feature(geom, style, "circle_point");
        self
    }

    pub fn add_polygon(&mut self, polygon: &GeoPolygon, style: Style) -> &mut Self {
        let geom = Geometry::from(polygon);
        self.push_feature(geom, style, "polygon");
        self
    }

    pub fn add_label(&mut self, coord: &Coord, text: &str, style: Style) -> &mut Self {
        let point = GeoPoint::from(*coord);
        let geom = Geometry::from(&point);
        self.push_feature_with_text(geom, style, "label", Some(text));
        self
    }

    fn push_feature(&mut self, geometry: Geometry, style: Style, layer_type: &str) {
        self.push_feature_with_text(geometry, style, layer_type, None);
    }

    fn push_feature_with_text(
        &mut self,
        geometry: Geometry,
        style: Style,
        layer_type: &str,
        text: Option<&str>,
    ) {
        if let Ok(serde_json::Value::Object(mut props)) = serde_json::to_value(style) {
            props.insert(
                "type".to_string(),
                serde_json::Value::String(layer_type.to_string()),
            );

            if let Some(t) = text {
                props.insert("text".to_string(), serde_json::Value::String(t.to_string()));
            }

            let feature = Feature {
                bbox: None,
                geometry: Some(geometry),
                id: None,
                properties: Some(props),
                foreign_members: None,
            };
            self.features.push(feature);
        }
    }

    pub fn write_html<P: AsRef<Path>>(&self, output_path: P) -> std::io::Result<()> {
        let feature_collection = FeatureCollection {
            bbox: None,
            features: self.features.clone(),
            foreign_members: None,
        };

        let geojson_string = feature_collection.to_string();

        let html_content = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>Geowide Debug Map</title>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="stylesheet" href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"/>
    <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"></script>
    <style>
        body {{ padding: 0; margin: 0; }}
        html, body, #map {{ height: 100vh; width: 100vw; }}
    </style>
</head>
<body>
    <div id="map"></div>
    <script>
        var map = L.map('map', {{ preferCanvas: true }});
        
        L.tileLayer('https://{{s}}.basemaps.cartocdn.com/dark_all/{{z}}/{{x}}/{{y}}{{r}}.png', {{
            maxZoom: 19,
            attribution: 'OpenStreetMap'
        }}).addTo(map);

        var geojsonData = {0};

        var geojsonLayer = L.geoJSON(geojsonData, {{
            style: function (feature) {{
                var props = feature.properties;
                if (props.type === 'line' || props.type === 'polygon') {{
                    return {{
                        color: props.stroke || '#3388ff',
                        weight: props.weight || 3,
                        opacity: props.opacity !== undefined ? props.opacity : 0.8,
                        fillColor: props.fill_color || '#3388ff',
                        fillOpacity: props.fill_opacity !== undefined ? props.fill_opacity : 0.2
                    }};
                }}
            }},
            pointToLayer: function (feature, latlng) {{
                var props = feature.properties;
                
                // Draw normal points
                if (props && props.type === 'circle_point') {{
                    return L.circleMarker(latlng, {{
                        radius: props.radius || 4,
                        color: props.stroke || '#000000',
                        weight: props.weight || 1,
                        opacity: props.opacity !== undefined ? props.opacity : 1.0,
                        fillColor: props.fill_color || '#ff0000',
                        fillOpacity: props.fill_opacity !== undefined ? props.fill_opacity : 0.9
                    }});
                }}
                
                // Draw text labels
                if (props && props.type === 'label') {{
                    var color = props.fill_color || props.stroke || '#ffffff';
                    var opacity = props.opacity !== undefined ? props.opacity : 1.0;
                    var fontSize = props.radius || 12; // Re-using radius for font-size!
                    var fontWeight = props.weight > 1 ? 'bold' : 'normal';

                    return L.marker(latlng, {{
                        icon: L.divIcon({{
                            className: '', // Critical: prevents Leaflet's default white-box background
                            html: `<div style="color: ${{color}}; opacity: ${{opacity}}; font-size: ${{fontSize}}px; font-weight: ${{fontWeight}}; white-space: nowrap; transform: translate(-50%, -50%); text-shadow: 1px 1px 2px #000;">${{props.text}}</div>`,
                            iconSize: null
                        }})
                    }});
                }}
                
                return L.marker(latlng);
            }}
        }}).addTo(map);

        if (geojsonLayer.getBounds().isValid()) {{
            map.fitBounds(geojsonLayer.getBounds());
        }} else {{
            map.setView([0, 0], 2);
        }}
    </script>
</body>
</html>
"#,
            geojson_string
        );

        let mut file = File::create(output_path)?;
        file.write_all(html_content.as_bytes())?;
        Ok(())
    }
}
