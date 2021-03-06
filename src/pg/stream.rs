use postgres::types::ToSql;
use std::io::{Error, ErrorKind};

use std::mem;

pub struct PGStream {
    cursor: String,
    pending: Option<Vec<u8>>,
    trans: postgres::Transaction<'static>,
    #[allow(dead_code)]
    conn: Box<r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>>
}

impl std::io::Read for PGStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut current = 0;

        while current < buf.len() {
            let mut write: Vec<u8> = Vec::new();

            if self.pending.is_some() {
                write = self.pending.clone().unwrap();
                self.pending = None;
            } else {
                let rows = match self.trans.query(&*format!("FETCH 1000 FROM {};", &self.cursor), &[]) {
                    Ok(rows) => rows,
                    Err(err) => {
                        return Err(Error::new(ErrorKind::Other, format!("{:?}", err)))
                    }
                };

                if !rows.is_empty() {
                    for row_it in 0..rows.len() {
                        let feat: String = rows.get(row_it).unwrap().get(0);
                        write.append(&mut feat.into_bytes().to_vec());
                        write.push(0x0A);
                    }
                }
            }

            if write.is_empty() {
                //No more data to fetch, close up shop
                break;
            } else if current + write.len() > buf.len() {
                //There is room to put a partial feature, saving the remaining
                //to the pending q and ending
                let length = buf.len();
                buf[current..].clone_from_slice(&write[0..(length - current)]);

                let pending = write[buf.len() - current..write.len()].to_vec();
                self.pending = Some(pending);

                current += buf.len() - current;

                break;
            } else {
                //There is room in the buff to print the whole feature
                //and iterate around to grab another

                buf[current..(write.len() + current)].clone_from_slice(&write[..]);

                current += write.len();
            }
        }

        Ok(current)
    }
}

impl PGStream {
    pub fn new(
        pg_conn: r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>,
        cursor: String,
        query: String,
        params: &[&(dyn ToSql + std::marker::Sync)]
    ) -> Result<Self, String> {
        let mut conn = Box::new(pg_conn);

        let mut trans: postgres::Transaction = unsafe {
            mem::transmute(conn.transaction().unwrap())
        };

        match trans.execute(&*query, params) {
            Ok(_) => {
                Ok(PGStream {
                    cursor,
                    pending: None,
                    trans,
                    conn
                })
            },
            Err(err) => Err(err.to_string())
        }
    }
}

