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

fn filter(feat: &Network) -> bool {
    match feat.props.get("highway") {
        Some(highway) => {
            let highway = highway.as_str().unwrap();

            let accepted = vec![
                "living_street",
                "motorway",
                "motorway_link",
                "primary",
                "primary_link",
                "residential",
                "road",
                "secondary",
                "secondary_link",
                "service",
                "tertiary",
                "tertiary_link",
                "trunk",
                "trunk_link",
                "unclassified"
            ];

            if !accepted.contains(&highway) {
                return true;
            }
        },
        None => {
            return true;
        }
    };

    match feat.props.get("surface") {
        Some(surface) => {
            let surface = surface.as_str().unwrap();

            let reject = vec![
                "dirt",
                "earth",
                "ground",
                "mud",
                "sand",
                "unpaved"
            ];

            if reject.contains(&surface) {
                return true;
            }
        },
        None => ()
    };

    false
}
