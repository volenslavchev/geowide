use std::collections::HashMap;

use geo::{Coord, LineString, Polygon, Rect};
use serde::Deserialize;

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct OsmBounds {
    pub maxlat: f64,
    pub maxlon: f64,
    pub minlat: f64,
    pub minlon: f64,
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct OsmPoint {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OsmPolygon(Vec<OsmPoint>);

pub type OsmElementId = u64;

#[derive(Deserialize, Debug, Clone)]
pub struct OsmElementWay {
    //pub r#type: String,
    pub id: OsmElementId,
    //pub bounds: OsmBounds,
    pub geometry: OsmPolygon,
    //pub nodes: Vec<OsmElementId>,
    pub tags: HashMap<String, String>
}

#[derive(Deserialize, Debug)]
pub struct OverpassResponse {
    // pub version: f32,
    // pub generator: String,
    pub elements: Vec<OsmElementWay>,
    pub remark: Option<String>,
}

impl From<Coord<f64>> for OsmPoint {
    fn from(value: Coord<f64>) -> Self {
        Self {
            lon: value.x,
            lat: value.y,
        }
    }
}

impl From<Rect<f64>> for OsmBounds {
    fn from(value: Rect<f64>) -> Self {
        Self {
            minlat: value.min().y,
            minlon: value.min().x,
            maxlat: value.max().y,
            maxlon: value.max().x,
        }
    }
}

impl From<OsmBounds> for Rect<f64> {
    fn from(value: OsmBounds) -> Self {
        Rect::new(
            Coord {
                x: value.minlon,
                y: value.minlat,
            },
            Coord {
                x: value.maxlon,
                y: value.maxlat,
            },
        )
    }
}

impl From<OsmPoint> for Coord<f64> {
    fn from(value: OsmPoint) -> Self {
        Coord {
            x: value.lon,
            y: value.lat,
        }
    }
}

impl From<OsmPolygon> for Polygon<f64> {
    fn from(value: OsmPolygon) -> Self {
        let points: Vec<Coord> = value.0.iter().cloned().map(|v| v.into()).collect();
        Polygon::new(LineString::new(points), vec![])
    }
}

impl From<OsmPolygon> for LineString<f64> {
    fn from(value: OsmPolygon) -> Self {
        let points: Vec<Coord> = value.0.iter().cloned().map(|v| v.into()).collect();
        LineString::new(points)
    }
}
