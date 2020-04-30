use super::AsTSV;
use std::convert::TryInto;

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
        String::from("")
    }
}
