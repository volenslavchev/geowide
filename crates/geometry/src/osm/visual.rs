use crate::osm::Path;
use geo::{MultiPoint, Point};
use geojson::{Feature, FeatureCollection, Geometry};
use serde_json::json;
use std::fs::File;
use std::io::Write;

pub fn visualize_paths(paths: &[Path], output_path: &str) -> std::io::Result<()> {
    let mut features = Vec::new();

    for path in paths {
        let line_geom = Geometry::from(&path.geometry);
        let line_feature = Feature {
            bbox: None,
            geometry: Some(line_geom),
            id: Some(geojson::feature::Id::Number(serde_json::Number::from(
                path.id,
            ))),
            properties: Some(
                json!({
                    "type": "road",
                    "stroke": "#3388ff",
                    "weight": 3
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
            foreign_members: None,
        };
        features.push(line_feature);

        let points: Vec<Point> = path
            .segments
            .0
            .iter()
            .map(|&coord| Point::from(coord))
            .collect();

        let multi_point = MultiPoint::new(points);
        let segments_geom = Geometry::from(&multi_point);

        let points_feature = Feature {
            bbox: None,
            geometry: Some(segments_geom),
            id: Some(geojson::feature::Id::Number(serde_json::Number::from(
                path.id,
            ))),
            properties: Some(
                json!({
                    "type": "shard",
                    "color": "#f03b20",
                    "radius": 4
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
            foreign_members: None,
        };
        features.push(points_feature);
    }

    let feature_collection = FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    };

    let geojson_string = feature_collection.to_string();

    let html_content = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Rust Path & Shard Visualizer</title>
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
        var map = L.map('map');
        
        L.tileLayer('https://{{s}}.basemaps.cartocdn.com/dark_all/{{z}}/{{x}}/{{y}}{{r}}.png', {{
            maxZoom: 19,
            attribution: 'OpenStreetMap'
        }}).addTo(map);

        var geojsonData = {};

        var geojsonLayer = L.geoJSON(geojsonData, {{
            style: function (feature) {{
                if (feature.properties.type === 'road') {{
                    return {{
                        color: feature.properties.stroke || '#3388ff',
                        weight: feature.properties.weight || 3,
                        opacity: 0.8
                    }};
                }}
            }},
            pointToLayer: function (feature, latlng) {{
                if (feature.properties.type === 'shard') {{
                    return L.circleMarker(latlng, {{
                        radius: feature.properties.radius || 4,
                        fillColor: feature.properties.color || '#ff0000',
                        color: '#000',
                        weight: 1,
                        opacity: 1,
                        fillOpacity: 0.9
                    }});
                }}
                return L.marker(latlng);
            }}
        }}).addTo(map);

        map.fitBounds(geojsonLayer.getBounds());
    </script>
</body>
</html>
"#,
        geojson_string
    );

    let mut file = File::create(output_path)?;
    file.write_all(html_content.as_bytes())?;

    println!("Map saved successfully to paths_debug_map.html");
    Ok(())
}
