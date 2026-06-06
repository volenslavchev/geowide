pub mod types;

use geo::{
    BooleanOps, Densify, Distance, Haversine, LineString, MultiLineString, MultiPolygon, Point,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::from_str;

use types::{OsmElementWay, OverpassResponse};

const SEGMENT_IDEAL_DISTANCE: f64 = 20.0;
const MIN_DIST_BTW_SEGMENTS: f64 = 12.0;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Path {
    pub id: u64,
    pub geometry: LineString,
    pub segments: LineString,
}

pub async fn get_all_paths(bbox: &MultiPolygon) -> Result<Vec<Path>, OverpassError> {
    let poly_string = bbox
        .iter()
        .flat_map(|poly| poly.exterior().coords())
        .map(|coord| format!("{:.6} {:.6} ", coord.y, coord.x))
        .collect::<String>();

    let res = overpass_api(&format!(
        "[out:json][timeout:120]; way['highway'](poly:\"{}\"); out geom;",
        poly_string.trim()
    ))
    .await?;

    let mut paths = res
        .elements
        .into_iter()
        .filter(osm_element_filter)
        .flat_map(|element| {
            let line: LineString = element.geometry.into();
            let multi_line = MultiLineString::new(vec![line]);
            let clipped_multi_line = bbox.clip(&multi_line, false);
            clipped_multi_line.into_iter().map(move |clipped_line| {
                let densified = Haversine.densify(&clipped_line, SEGMENT_IDEAL_DISTANCE);
                Path {
                    id: element.id,
                    geometry: clipped_line,
                    segments: LineString(densified.0),
                }
            })
        })
        .collect::<Vec<_>>();

    let mut global_saved_points: Vec<Point> = Vec::new();

    for path in paths.iter_mut() {
        let mut clean_segments: Vec<geo::Coord> = Vec::new();

        for &coord in path.segments.0.iter() {
            let candidate_point = Point::from(coord);
            let is_too_close = global_saved_points.iter().any(|saved_pt| {
                Haversine.distance(candidate_point, saved_pt.clone()) < MIN_DIST_BTW_SEGMENTS
            });

            if !is_too_close {
                clean_segments.push(coord);
                global_saved_points.push(candidate_point);
            }
        }
        path.segments = LineString::new(clean_segments);
    }

    Ok(paths)
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

#[derive(Clone, Debug, Serialize)]
pub enum OverpassError {
    UnknownServerError,
    ServerBusy,
    QueryTimeout,
    ServerOutOfMemory,
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