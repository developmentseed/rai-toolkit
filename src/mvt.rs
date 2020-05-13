#[allow(clippy::excessive_precision)]

pub use super::grid::{Grid};

pub fn tile(pool: &r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>, iso: &String, z: u8, x: u32, y: u32) -> Result<Vec<u8>, String> {
    let grid = Grid::web_mercator();
    let bbox = grid.tile_extent(z, x, y);

    match pool.get().unwrap().query(format!("
        SELECT
            ST_AsMVT(q, 'data', 4096, 'geom')
        FROM (
            SELECT
                id,
                pop,
                coverage,
                ST_AsMVTGeom(geom, ST_Transform(ST_MakeEnvelope($1, $2, $3, $4, $5), 4326), 4096, 256, false) AS geom
            FROM
                country_{iso}.{iso}_geom
            WHERE
                ST_Intersects(geom, ST_Transform(ST_MakeEnvelope($1, $2, $3, $4, $5), 4326))
        ) q
    ", iso = iso).as_str(), &[&bbox.minx, &bbox.miny, &bbox.maxx, &bbox.maxy, &grid.srid]) {
        Ok(res) => {
            let tile: Vec<u8> = res.get(0).unwrap().get(0);
            Ok(tile)
        },
        Err(err) => Err(err.to_string())
    }
}
