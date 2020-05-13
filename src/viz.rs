use crate::mvt;
use actix_web::{web, App, HttpResponse, HttpServer, middleware};
use futures::Future;

pub fn main(pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>, args: &clap_v3::ArgMatches) {
    let iso = args.value_of("iso").unwrap().to_string().to_lowercase();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::NormalizePath)
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .data(pool.clone())
            .service(
                actix_files::Files::new("/", String::from("./web/dist/"))
                .index_file("index.html")
            )
            .service(web::scope("tiles")
                .service(web::resource("{z}/{x}/{y}")
                    .route(web::get().to_async(mvt_get))
                )
            )
        })
            .workers(12 as usize)
            .bind(format!("0.0.0.0:{}", 4001).as_str())
            .unwrap()
            .run()
            .unwrap();
}

fn mvt_get(
    db: web::Data<r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>>,
    path: web::Path<(u8, u32, u32)>
) -> impl Future<Item = HttpResponse, Error = String> {
    web::block(move || {
        let z = path.0;
        let x = path.1;
        let y = path.2;

        if z > 17 { return Err(String::from("Tile Not Found")); }

        Ok(mvt::tile(&db, z, x, y)?)
    }).then(|res: Result<Vec<u8>, actix_threadpool::BlockingError<String>>| match res {
        Ok(tile) => {
            Ok(HttpResponse::build(actix_web::http::StatusCode::OK)
               .content_type("application/x-protobuf")
               .content_length(tile.len() as u64)
               .body(tile))
        },
        Err(err) => Err(err.to_string())
    })
}
