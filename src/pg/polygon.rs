use postgres::Client;
use std::io::Read;
use super::{Table, InputTable};

///
/// Polygon table are special in that they don't make assumptions about the underlying
/// data. They can be any one of a number of types - building polys, parcels, places
///
pub struct Polygon {
    name: String
}

impl Polygon {
    pub fn new(name: impl ToString) -> Self {
        Polygon {
            name: name.to_string()
        }
    }
}

impl Table for Polygon {
    fn create(&self, conn: &mut Client) {
        conn.execute(r#"
             CREATE EXTENSION IF NOT EXISTS POSTGIS
        "#, &[]).unwrap();

        conn.execute(format!(r#"
            DROP TABLE IF EXISTS {};
        "#, &self.name).as_str(), &[]).unwrap();

        conn.execute(format!(r#"
            CREATE UNLOGGED TABLE {} (
                id BIGINT,
                props JSONB,
                geom GEOMETRY(MultiPolygon, 4326)
            )
        "#, &self.name).as_str(), &[]).unwrap();
    }

    fn max(&self, db: &mut Client) -> Option<i64> {
        let max: Option<i64> = match db.query(format!("
            SELECT
                MAX(id)
            FROM
                {}
        ", &self.name).as_str(), &[]) {
            Err(err) => panic!("{}", err),
            Ok(res) => res.get(0).unwrap().get(0)
        };

        max
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn props(&self, _db: &mut Client, _id: i64) -> serde_json::Map<String, serde_json::Value> {
        serde_json::Map::new()
    }

    fn count(&self, conn: &mut Client) -> i64 {
        match conn.query(format!(r#"
            SELECT count(*) FROM {}
        "#, &self.name).as_str(), &[]) {
            Ok(res) => {
                let cnt: i64 = res.get(0).unwrap().get(0);
                cnt
            },
            _ => 0
        }
    }

    fn index(&self, conn: &mut Client) {
        conn.execute(format!(r#"
            CREATE INDEX {name}_idx ON {name} (id);
        "#, name = &self.name).as_str(), &[]).unwrap();

        conn.execute(format!(r#"
            CREATE INDEX {name}_gix ON {name} USING GIST (geom);
        "#, name = &self.name).as_str(), &[]).unwrap();
    }
}

impl InputTable for Polygon {
    fn input(&self, conn: &mut Client, mut data: impl Read) {
        let mut stmt = conn.copy_in(format!(r#"
            COPY {} (
                props,
                geom
            )
            FROM STDIN
            WITH (
                FORMAT CSV,
                NULL '',
                DELIMITER E'\t',
                QUOTE E'\b'
            )
        "#, &self.name).as_str()).unwrap();

        std::io::copy(&mut data, &mut stmt).unwrap();
        stmt.finish().unwrap();

        conn.execute(format!(r#"
            UPDATE {name}
                SET geom = ST_CollectionExtract(ST_MakeValid(geom), 3)
        "#, name = &self.name).as_str(), &[]).unwrap();
    }
}
