extern crate total_recall;

use actix_files::Files;
use actix_web::{
    web::{get, post},
    App, HttpServer,
};
use diesel::r2d2::{ConnectionManager, Pool};
use std::sync::Arc;
use structopt::StructOpt;

use total_recall::{
    db::DBConnection,
    graphql::{mutations::Mutation, query::Query, GQLContext, Schema},
    service::{
        endpoints::{graphiql, graphql, login},
        AppState,
    },
};

#[derive(Debug, StructOpt)]
#[structopt(name = "total_recall")]
struct Opt {
    #[structopt(short = "u", long = "db-url")]
    database_url: String,
    #[structopt(short = "s", long = "socket", default_value = "127.0.0.1:8000")]
    socket: String,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    let manager = ConnectionManager::<DBConnection>::new(opt.database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to initialize connection pool");

    let query = Query::<GQLContext<DBConnection>>::default();
    let mutation = Mutation::<GQLContext<DBConnection>>::default();
    let schema = Schema::new(query, mutation);

    let schema = Arc::new(schema);
    let pool = Arc::new(pool);
    let data = AppState { schema, pool };

    let url = opt.socket;

    println!("Total recall running at: http://{}", url);

    HttpServer::new(move || {
        App::new()
            .data(data.clone())
            .route("/login", post().to(login))
            .route("/graphql", post().to(graphql))
            .route("/graphiql", get().to(graphiql))
            .service(Files::new("/", "./static").index_file("index.html"))
    })
    .bind(&url)
    .expect("Failed to start Total Recall")
    .run()
    .await
}
