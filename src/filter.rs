use geojson::{Feature, GeoJson};
use std::io::Write;
use crate::stream::{GeoStream, NetStream};

pub fn main(args: &clap_v3::ArgMatches) {
    let osm_src = args.value_of("OSM").unwrap().to_string();

    for feat in NetStream::new(GeoStream::new(Some(osm_src)), None) {
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
                    continue;
                }
            },
            None => continue
        };

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
