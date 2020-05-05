use postgres::Client;
use geojson::GeoJson;
use std::io::Read;
use super::{Table, InputTable};
use std::collections::HashMap;

pub struct Country {
    name: String
}

impl Country {
    pub fn new() -> Self {
        Country {
            name: String::from("country")
        }
    }
}

impl Table for Country {
    fn create(&self, conn: &mut Client) {
        conn.execute(r#"
             CREATE EXTENSION IF NOT EXISTS POSTGIS
        "#, &[]).unwrap();

        conn.execute(format!("
            DROP TABLE IF EXISTS {};
        ", &self.name).as_str(), &[]).unwrap();

        conn.execute(format!("
            CREATE UNLOGGED TABLE {} (
                id      BIGSERIAL,
                name    TEXT,
                iso     TEXT,
                geom    GEOMETRY(MultiPolygon, 4326)
            )
        ", &self.name).as_str(), &[]).unwrap();

        let countries = reqwest::blocking::get("https://github.com/datasets/geo-countries/raw/master/data/countries.geojson").unwrap()
            .text().unwrap();

        let countries = match countries.parse::<GeoJson>().unwrap() {
            GeoJson::FeatureCollection(fc) => fc.features,
            _ => panic!("Countries must be a GeoJSON feature collection")
        };

        for country in countries {
            let props = match country.properties {
                Some(props) => props,
                None => panic!("Country Feature is missing properties")
            };

            let name: String = match props.get("ADMIN") {
                Some(name) => match name.as_str() {
                    Some(name) => name.to_string(),
                    None => panic!("Country Feature ADMIN property must be string")
                },
                None => panic!("Country Feature is missing ADMIN property")
            };

            let iso: String = match props.get("ISO_A2") {
                Some(iso) => match iso.as_str() {
                    Some(iso) => iso.to_string(),
                    None => panic!("Country Feature ISO_A2 property must be string")
                },
                None => panic!("Country Feature is missing ISO_A2 property")
            };

            let geom: String = match country.geometry {
                Some(geom) => geom.to_string(),
                None => panic!("Country Feature is missing geometry")
            };

            conn.query(r#"
                INSERT INTO country (
                    name,
                    iso,
                    geom
                ) VALUES (
                    $1,
                    $2,
                    ST_SetSRID(ST_Multi(ST_GeomFromGeoJSON($3)), 4326)
                )
            "#, &[
                &name,
                &iso,
                &geom
            ]).unwrap();
        }
    }

    fn count(&self, conn: &mut Client) -> i64 {
        match conn.query(format!("
            SELECT count(*) FROM {}
        ", &self.name).as_str(), &[]) {
            Ok(res) => {
                let cnt: i64 = res.get(0).unwrap().get(0);
                cnt
            },
            _ => 0
        }
    }

    fn index(&self, conn: &mut Client) {
        conn.execute(format!("
            CREATE INDEX {name}_gix ON {name} USING GIST (geom);
        ", name = &self.name).as_str(), &[]).unwrap();
    }
}
