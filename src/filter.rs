use geojson::{Feature, GeoJson};
use std::io::Write;
use crate::stream::{GeoStream, NetStream};
use crate::types::Network;

pub fn main(args: &clap_v3::ArgMatches) {
    let osm_src = args.value_of("OSM").unwrap().to_string();

    for feat in NetStream::new(GeoStream::new(Some(osm_src)), None) {
        if filter(&feat) {
            continue;
        }

        let f = GeoJson::Feature(Feature {
            id: None,
            bbox: None,
            geometry: Some(geojson::Geometry::new(geojson::Value::from(&feat.geom))),
            properties: Some(feat.props),
            foreign_members: None,
        }).to_string();

        std::io::stdout().write_all(format!("{}\n", f).as_bytes()).unwrap();
    }
}

pub fn conditional_highway() -> Vec<&'static str> {
    vec![
        "living_street",
        "unclassified",
        "residential",
        "service"
    ]
}

pub fn default_highway() -> Vec<&'static str> {
    vec![
        "motorway",
        "motorway_link",
        "primary",
        "primary_link",
        "secondary",
        "secondary_link",
        "tertiary",
        "tertiary_link",
        "trunk",
        "trunk_link"
    ]
}

pub fn pref_surface() -> Vec<&'static str> {
    vec![
        "paved",
        "asphalt",
        "concrete",
        "concrete:lanes",
        "concrete:plates",
        "paving_stones",
        "sett",
        "unhewn_cobblestone",
        "cobblestone"
    ]
}

pub fn reject_surface() -> Vec<&'static str> {
    vec![
        "dirt",
        "earth",
        "ground",
        "mud",
        "sand",
        "grass",
        "unpaved",
        "compacted",
        "fine_gravel",
        "gravel",
        "pebblestone"
    ]
}

fn filter(feat: &Network) -> bool {
    let highway = match feat.props.get("highway") {
        Some(highway) => highway.as_str().unwrap(),
        None => { return true; }
    };

    let surface = match feat.props.get("surface") {
        Some(surface) => surface.as_str().unwrap(),
        None => ""
    };

    if reject_surface().contains(&surface) {
        return true;
    } else if conditional_highway().contains(&highway) && !pref_surface().contains(&surface) {
        return true;
    } else if !default_highway().contains(&highway) && !conditional_highway().contains(&highway) {
        return true;
    }

    false
}
