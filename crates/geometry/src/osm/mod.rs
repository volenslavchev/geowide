pub mod types;

use geo::{
    Bearing, BooleanOps, Destination, Distance, Geometry, Haversine, LineString, MultiLineString,
    MultiPolygon, Point,
};
use reqwest::StatusCode;
use rstar::{AABB, RTree};
use serde::{Deserialize, Serialize};
use serde_json::from_str;

use types::OverpassResponse;

use crate::osm::types::OsmElementWay;

pub struct OsmConfig {
    pub segment_ideal_distance: f64,
    pub segment_min_distance: f64,
}

#[derive(Debug)]
pub(crate) struct MultiLinePath {
    pub id: u64,
    pub geometry: MultiLineString,
    pub segments: MultiLineString,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Path {
    pub id: u64,
    pub geometry: LineString,
    pub segments: LineString,
}

pub fn clip_paths<'a>(
    bbox: &MultiPolygon,
    elements: impl Iterator<Item = &'a MultiLinePath>,
) -> Vec<Path> {
    elements
        .flat_map(|element| {
            let clipped_segments = bbox.clip(&element.segments, false);
            let clipped_geometry = bbox.clip(&element.geometry, false);

            clipped_geometry
                .0
                .into_iter()
                .zip(clipped_segments.0.into_iter())
                .map(move |(geometry, segments)| Path {
                    id: element.id,
                    geometry,
                    segments,
                })
        })
        .collect()
}

pub async fn get_all_paths(
    bbox: &MultiPolygon,
    config: &OsmConfig,
) -> Result<Vec<MultiLinePath>, OverpassError> {
    let poly_string = bbox
        .iter()
        .flat_map(|poly| poly.exterior().coords())
        .map(|coord| format!("{:.6} {:.6} ", coord.y, coord.x))
        .collect::<String>();

    let mut paths: Vec<MultiLinePath> = overpass_api(&format!(
        "[out:json][timeout:120]; way['highway'](poly:\"{}\"); out geom;",
        poly_string.trim()
    ))
    .await?
    .elements
    .into_iter()
    .filter(|el| osm_element_filter(el))
    .map(|element| {
        let geometry: MultiLineString = element.geometry.into();
        let segments = resample_multiline(&geometry, config.segment_ideal_distance);
        MultiLinePath {
            id: element.id,
            geometry,
            segments,
        }
    })
    .collect();

    let mut rtree: RTree<[f64; 2]> = RTree::new();

    for path in &mut paths {
        let mut clean_lines = Vec::with_capacity(path.segments.0.len());

        for line in &path.segments.0 {
            let mut clean_coords = Vec::new();

            for &coord in &line.0 {
                let candidate_point = Point::from(coord);

                // Pre-filter bounding box
                let lat_buffer = config.segment_min_distance / 111_111.0;
                let lon_buffer = lat_buffer / coord.y.to_radians().cos().abs();

                let envelope = AABB::from_corners(
                    [coord.x - lon_buffer, coord.y - lat_buffer],
                    [coord.x + lon_buffer, coord.y + lat_buffer],
                );

                // Use `.any()` to implicitly short-circuit the search if a point is too close
                let too_close = rtree
                    .locate_in_envelope_intersecting(envelope)
                    .any(|nearby_pt| {
                        let saved_pt = Point::new(nearby_pt[0], nearby_pt[1]);
                        Haversine.distance(candidate_point, saved_pt) < config.segment_min_distance
                    });

                if !too_close {
                    clean_coords.push(coord);
                    rtree.insert([coord.x, coord.y]);
                }
            }
            clean_lines.push(LineString::new(clean_coords));
        }
        path.segments = MultiLineString::new(clean_lines);
    }

    Ok(paths)
}


pub fn resample_multiline(multi_line: &MultiLineString, interval_meters: f64) -> MultiLineString {
    let resampled_lines: Vec<LineString> = multi_line
        .0
        .iter()
        .map(|line| resample_line(line, interval_meters))
        .collect();

    MultiLineString::new(resampled_lines)
}

fn resample_line(line: &LineString, interval_meters: f64) -> LineString {
    let coords = &line.0;
    if coords.len() < 2 {
        return line.clone();
    }

    let mut resampled_coords = Vec::with_capacity(coords.len());
    resampled_coords.push(coords[0]);

    let mut current_pos = Point::from(coords[0]);
    let mut target_idx = 1;

    while target_idx < coords.len() {
        let target_pos = Point::from(coords[target_idx]);
        let dist_to_target = Haversine.distance(current_pos, target_pos);

        // Safe unwrap because we always push coords[0] at the start
        let last_dropped = Point::from(*resampled_coords.last().unwrap());
        let dist_from_last = Haversine.distance(last_dropped, current_pos);

        let remaining_needed = interval_meters - dist_from_last;

        if dist_to_target >= remaining_needed {
            let bearing = Haversine.bearing(current_pos, target_pos);
            let new_point = Haversine.destination(current_pos, bearing, remaining_needed);

            resampled_coords.push(new_point.into());
            current_pos = new_point;
        } else {
            current_pos = target_pos;
            target_idx += 1;
        }
    }

    LineString::new(resampled_coords)
}

#[derive(Deserialize, Debug)]
struct NominatimResult {
    geojson: geojson::Geometry,
}

pub async fn search_coordinates(query: &str) -> anyhow::Result<Geometry> {
    let client = reqwest::Client::new();

    let res: Vec<NominatimResult> = client
        .get(format!(
            "https://nominatim.openstreetmap.org/search?q={}&format=json&limit=1&polygon_geojson=1",
            query
        ))
        .header("User-Agent", "Geowide/1.0")
        .send()
        .await?
        .json()
        .await?;

    let city_data = res
        .into_iter()
        .next()
        .ok_or(anyhow::anyhow!("City not found in OpenStreetMap!"))?;

    Ok(city_data.geojson.try_into()?)
}

fn osm_element_filter(el: &OsmElementWay) -> bool {
    let highway_tag = el.tags.get("highway").map(|s| s.as_str()).unwrap_or("");
    match highway_tag {
        "motorway" | "motorway_link" | "trunk" | "trunk_link" | "primary" | "primary_link"
        | "secondary_link" | "tertiary" | "tertiary_link" | "unclassified" | "residential"
        | "pedestrian" | "footway" => true,
        "service" => matches!(
            el.tags.get("access").map(|v| v.as_str()),
            Some("public") | None
        ),
        _ => false,
    }
}

#[derive(Clone, Debug, Serialize, thiserror::Error)]
pub enum OverpassError {
    #[error("Unknown server error")]
    UnknownServerError,
    #[error("Overpass server is busy")]
    ServerBusy,
    #[error("Query timeout")]
    QueryTimeout,
    #[error("Server out of memory")]
    ServerOutOfMemory,
    #[error("Overpass query parsing")]
    ParseError,
}

async fn overpass_api(query: &str) -> Result<OverpassResponse, OverpassError> {
    let body = [("data", query)];
    let client = reqwest::Client::new();

    let res = client
        .post("https://overpass-api.de/api/interpreter")
        .form(&body)
        .header("Accept", "application/json")
        .header("User-Agent", "Geowide/1.0")
        .send()
        .await
        .map_err(|_| OverpassError::UnknownServerError)?;

    let status = res.status();

    let text_body = res
        .text()
        .await
        .map_err(|_| OverpassError::UnknownServerError)?;

    if !status.is_success() {
        return match status {
            StatusCode::TOO_MANY_REQUESTS => Err(OverpassError::ServerBusy),
            StatusCode::GATEWAY_TIMEOUT => Err(OverpassError::QueryTimeout),
            StatusCode::BAD_REQUEST => {
                println!("Overpass Syntax Error: {}", text_body);
                Err(OverpassError::ParseError)
            }
            _ => Err(OverpassError::UnknownServerError),
        };
    }

    let response =
        from_str::<OverpassResponse>(&text_body).map_err(|_| OverpassError::ParseError)?;

    if let Some(remark) = &response.remark {
        if remark.contains("timed out") {
            return Err(OverpassError::QueryTimeout);
        } else if remark.contains("out of memory") || remark.contains("exceeds the allowed limit") {
            return Err(OverpassError::ServerOutOfMemory);
        } else {
            return Err(OverpassError::UnknownServerError);
        }
    }

    Ok(response)
}
