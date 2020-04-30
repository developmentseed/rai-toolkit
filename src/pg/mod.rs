use std::iter::Iterator;
use postgres::Client;
use std::io::Read;
use std::mem;
use serde_json::Value;

pub mod network;
pub use self::network::Network;

pub trait Table {
    fn create(&self, conn: &mut Client);
    fn count(&self, conn: &mut Client) -> i64;
    fn index(&self, conn: &mut Client);
}

///
/// Tables which are designed to accecpt tabular input via a Read trait
/// will implement the InputTable Property
///
pub trait InputTable {
    fn input(&self, conn: &mut Client, data: impl Read);
    fn seq_id(&self, conn: &mut Client);
}

///
/// Relatively limited cursor wrapper that will allow a cursor to be
/// created that returns a single Serde_Json::Value field
///
pub struct Cursor {
    pub fetch: i64,
    pub query: String,
    trans: postgres::Transaction<'static>,
    #[allow(dead_code)]
    conn: Box<postgres::Client>,
    cache: Vec<Value>
}

impl Cursor {
    pub fn new(conn: Client, query: String) -> Result<Self, String> {
        let fetch = 1000;

        let mut pg_conn = Box::new(conn);

        let mut trans: postgres::Transaction = unsafe {
            mem::transmute(match pg_conn.transaction() {
                Ok(trans) => trans,
                Err(err) => {
                    return Err(err.to_string());
                }
            })
        };

        match trans.execute(format!(r#"
            DECLARE next_cursor CURSOR FOR {}
        "#, &query).as_str(), &[]) {
            Err(err) => {
                return Err(err.to_string());
            },
            _ => ()
        };

        Ok(Cursor {
            fetch: fetch,
            conn: pg_conn,
            trans: trans,
            query: query,
            cache: Vec::with_capacity(fetch as usize)
        })
    }
}

impl Iterator for Cursor {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.cache.is_empty() {
            return self.cache.pop()
        }

        let rows = match self.trans.query(format!(r#"
            FETCH {} FROM next_cursor
        "#, &self.fetch).as_str(), &[]) {
            Ok(rows) => rows,
            Err(err) => panic!("Fetch Error: {}", err.to_string())
        };

        // Cursor is finished
        if rows.is_empty() {
            return None;
        } else {
            self.cache = rows.iter().map(|row| {
                let json: Value = row.get(0);
                json
            }).collect();

            return self.cache.pop();
        }
    }
}

