use postgres::Client;
use std::io::Read;
use super::{Table, InputTable};

#[derive(Clone)]
pub struct Network {
    name: String
}

impl Network {
    pub fn new(name: impl ToString) -> Self {
        Network {
            name: name.to_string()
        }

    }

    pub fn props(&self, db: &mut Client, id: i64) -> serde_json::Map<String, serde_json::Value> {
       let props: serde_json::Value = match db.query("
            SELECT
                props
            FROM
                master
            WHERE
                id = $1
        ", &[&id]) {
            Err(err) => panic!("{}", err),
            Ok(res) => res.get(0).unwrap().get(0)
        };

        let props = match props {
            serde_json::Value::Object(props) => props,
            _ => panic!("props must be json object")
        };

        props
    }

    pub fn max(&self, db: &mut Client) -> Option<i64> {
        let max: Option<i64> = match db.query("
            SELECT
                MAX(id)
            FROM
                master
        ", &[]) {
            Err(err) => panic!("{}", err),
            Ok(res) => res.get(0).unwrap().get(0)
        };

        max
    }

    pub fn seq(&self, db: &mut Client) {
        db.execute(format!("
            ALTER TABLE {}
                ADD COLUMN name JSONB
        ", self.name).as_str(), &[]).unwrap();

        db.execute(format!("
            DROP SEQUENCE IF EXISTS {}_seq;
        ", self.name).as_str(), &[]).unwrap();

        db.execute(format!("
            CREATE SEQUENCE {}_seq;
        ", self.name).as_str(), &[]).unwrap();

        db.execute(format!("
            ALTER TABLE {name}
                ALTER COLUMN id
                    SET DEFAULT nextval('{name}_seq');
        ", name = self.name).as_str(), &[]).unwrap();

        db.execute(format!("
            ALTER SEQUENCE {name}_seq OWNED BY {name}.id;
        ", name = self.name).as_str(), &[]).unwrap();

        db.execute(format!("
            UPDATE {name}
                SET
                    id = nextval('{name}_seq')
        ", name = self.name).as_str(), &[]).unwrap();
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
            CREATE INDEX {}_idx ON {} (id);
        ", &self.name.replace(".", "_"), &self.name).as_str(), &[]).unwrap();

        conn.execute(format!("
            CREATE INDEX {}_gix ON {} USING GIST (geom);
        ", &self.name.replace(".", "_"), &self.name).as_str(), &[]).unwrap();

        conn.execute(format!("
            CLUSTER {} USING {}_idx;
        ", &self.name, &self.name.replace(".", "_")).as_str(), &[]).unwrap();

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
        stmt.finish().unwrap();
    }
}
