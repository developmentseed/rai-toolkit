use super::AsTSV;
use std::convert::TryInto;
use postgis::ewkb::EwkbWrite;

pub struct Network {
    pub id: Option<i64>,
    pub props: serde_json::Map<String, serde_json::Value>,
    pub geom: geo::MultiLineString<f64>
}

impl Network {
    pub fn new(feat: geojson::GeoJson) -> Result<Self, String> {
        let feat = match feat {
            geojson::GeoJson::Feature(feat) => feat,
            _ => { return Err(String::from("Not a GeoJSON Feature")); }
        };

        let props = match feat.properties {
            Some(props) => props,
            None => { return Err(String::from("Feature has no properties")); }
        };

        let geom = match feat.geometry {
            Some(geom) => match geom.value {
                geojson::Value::LineString(ln) => geojson::Value::MultiLineString(vec![ln]),
                geojson::Value::MultiLineString(mln) => geojson::Value::MultiLineString(mln),
                _ => { return Err(String::from("Network must have (Multi)LineString geometry")); }
            },
            None => { return Err(String::from("Network must have geometry")); }
        };

        let geom: geo::MultiLineString<f64> = match geom.try_into() {
            Ok(geom) => geom,
            Err(err) => {
                return Err(format!("Invalid GeoJSON geometry: {}", err));
            }
        };

        Ok(Network {
            id: None,
            props,
            geom
        })
    }
}

impl AsTSV for Network {
    fn as_tsv(self) -> String {
        let mut twkb = postgis::twkb::MultiLineString {
            lines: Vec::with_capacity(self.geom.0.len()),
            ids: None
        };

        for ln in self.geom {
            let mut line = postgis::twkb::LineString {
                points: Vec::with_capacity(ln.0.len())
            };

            for pt in ln {
                line.points.push(postgis::twkb::Point {
                    x: pt.x,
                    y: pt.y
                });
            }

            twkb.lines.push(line);
        }

        let geom = postgis::ewkb::EwkbMultiLineString {
            geom: &twkb,
            srid: Some(4326),
            point_type: postgis::ewkb::PointType::Point
        }.to_hex_ewkb();

        format!("{props}\t{geom}\n",
            props = serde_json::value::Value::from(self.props),
            geom = geom
        )
    }
}
