use serde_json::json;
use crate::mvt;
use actix_web::{web, App, HttpResponse, HttpServer, middleware, web::Json};

#[derive(Debug, Clone)]
struct Country(String);

pub fn main(pool: r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>, args: &clap_v3::ArgMatches) {
    let iso = Country(args.value_of("iso").unwrap().to_string().to_lowercase());

    let token = match std::env::var("MAPBOX_TOKEN") {
        Ok(tk) => tk,
        Err(e) => panic!("MAPBOX_TOKEN environemnt variable required"),
    };

    println!("\nPoint your browser to:");
    println!("http://localhost:4001\n");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::NormalizePath)
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .data(pool.clone())
            .data(token.clone())
            .data(iso.clone())
            .service(web::scope("tiles")
                .service(web::resource("")
                    .route(web::get().to(map_get))
                )
                .service(web::resource("{z}/{x}/{y}")
                    .route(web::get().to(mvt_get))
                )
            )
            .service(
                actix_files::Files::new("/", String::from("./web/dist/"))
                .index_file("index.html")
            )
        })
            .workers(12 as usize)
            .bind(format!("0.0.0.0:{}", 4001).as_str())
            .unwrap()
            .run()
            .unwrap();
}

fn map_get(
    db: web::Data<r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>>,
    iso: web::Data<Country>,
    token: web::Data<String>
) -> Json<serde_json::Value> {

    let extent = match db.get().unwrap().query(format!(r#"
        SELECT
            regexp_replace(regexp_replace(regexp_replace(ST_Extent(geom)::TEXT, 'BOX\(', '['), '\)', ']'), ' ', ',', 'g')::JSON
        FROM
            country_{iso}.{iso}_geom;
    "#, iso = &iso.0).as_str(), &[]) {
        Err(err) => panic!(err.to_string()),
        Ok(rows) => {
            let bounds: serde_json::Value = rows.get(0).unwrap().get(0);
            bounds
        }
    };

    Json(json!({
        "bounds": extent,
        "token": token.as_str()
    }))
}

fn mvt_get(
    db: web::Data<r2d2::Pool<r2d2_postgres::PostgresConnectionManager<postgres::NoTls>>>,
    path: web::Path<(u8, u32, u32)>
) -> HttpResponse {
    let z = path.0;
    let x = path.1;
    let y = path.2;

    if z > 17 {
        let body = String::from("Tile not found");
        return HttpResponse::build(actix_web::http::StatusCode::NOT_FOUND)
           .content_type("text/plain")
           .content_length(body.len() as u64)
           .body(body);
    }

    let tile = match mvt::tile(&db, z, x, y) {
        Ok(tile) => tile,
        Err(err) => {
            println!("{}", err);

            let body: String = err.to_string();

            return HttpResponse::build(actix_web::http::StatusCode::NOT_FOUND)
               .content_type("text/plain")
               .content_length(body.len() as u64)
               .body(body);
        }
    };

    HttpResponse::build(actix_web::http::StatusCode::OK)
       .content_type("application/x-protobuf")
       .content_length(tile.len() as u64)
       .body(tile)
}
