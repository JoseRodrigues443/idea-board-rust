#[macro_use]
extern crate actix_web;
#[macro_use]
extern crate diesel;

extern crate dotenv;

use dotenv::dotenv;
use std::{env, io};

use actix_web::{middleware, App, HttpServer};
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::{Pool, PooledConnection};

mod constants;
mod like;
mod response;
mod schema;
mod idea;

use crate::constants::{ADDRESS};

pub type DBPool = Pool<ConnectionManager<PgConnection>>;
pub type DBPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv().ok(); // load from .env file
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    // set up database connection pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    HttpServer::new(move || {
        App::new()
            // Set up DB pool to be used with web::Data<Pool> extractor
            .data(pool.clone())
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
            // register HTTP requests handlers
            .service(idea::list)
            .service(idea::get)
            .service(idea::create)
            .service(idea::delete)
            .service(like::list)
            .service(like::plus_one)
            .service(like::minus_one)
    })
    .bind(ADDRESS)?
    .run()
    .await
}
