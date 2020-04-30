use postgres::Client;
use std::io::Read;
use super::{Table, InputTable};

pub struct Network {
    name: String
}

impl Network {
    pub fn new(name: impl ToString) -> Self {
        Network {
            name: name.to_string()
        }
    }
}

impl Table for Network {
    fn create(&self, conn: &mut Client) {
        conn.execute(r#"
             CREATE EXTENSION IF NOT EXISTS POSTGIS
        "#, &[]).unwrap();

        conn.execute(format!("
            DROP TABLE IF EXISTS {};
        ", &self.name).as_str(), &[]).unwrap();

        conn.execute(format!("
            CREATE UNLOGGED TABLE {} (
                id BIGINT,
                props JSONB,
                geom GEOMETRY(MultiLineString, 4326)
            )
        ", &self.name).as_str(), &[]).unwrap();
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
            ALTER TABLE {name}
                ALTER COLUMN geom
                TYPE GEOMETRY(MULTILINESTRINGZ, 4326)
                USING ST_GeomFromEWKT(Regexp_Replace(ST_AsEWKT(geom)::TEXT, '(?<=\\d)(?=[,)])', ' '||id, 'g'))
        ", name = &self.name).as_str(), &[]).unwrap();

        conn.execute(format!("
            CREATE INDEX {name}_idx ON {name} (id);
        ", name = &self.name).as_str(), &[]).unwrap();

        conn.execute(format!("
            CREATE INDEX {name}_gix ON {name} USING GIST (geom);
        ", name = &self.name).as_str(), &[]).unwrap();

        conn.execute(format!("
            CLUSTER {name} USING {name}_idx;
        ", name = &self.name).as_str(), &[]).unwrap();

        conn.execute(format!("
            ANALYZE {name};
        ", name = &self.name).as_str(), &[]).unwrap();
    }
}

impl InputTable for Network {
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
    }

    fn seq_id(&self, conn: &mut Client) {
        conn.execute(format!("
            DROP SEQUENCE IF EXISTS {name}_seq;
        ", name = &self.name).as_str(), &[]).unwrap();

        conn.execute(format!("
            CREATE SEQUENCE {name}_seq;
        ", name = &self.name).as_str(), &[]).unwrap();

        conn.execute(format!("
            UPDATE $1
                SET id = nextval('{name}_seq');
        ", name = &self.name).as_str(), &[]).unwrap();
    }
}
