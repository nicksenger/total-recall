extern crate total_recall;

use std::{
    io::Result,
    path::PathBuf,
    sync::Arc,
};

use actix_files::NamedFile;
use actix_web::{
    middleware::Logger,
    web::{get, post},
    App, HttpRequest, HttpServer,
};
use diesel::r2d2::{ConnectionManager, Pool};
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

async fn index(req: HttpRequest) -> Result<NamedFile> {
    let mut path = req.match_info().query("path").split("/").peekable();
    match path.peek() {
        Some(&"login") | Some(&"register") | Some(&"manual") | Some(&"study") | Some(&"cards")
        | Some(&"sets") | Some(&"user") | None => Ok(NamedFile::open::<PathBuf>(
            "./static/index.html".parse().unwrap(),
        )?),
        Some(_) => Ok(NamedFile::open::<PathBuf>(
            format!("./static/{}", path.collect::<Vec<&str>>().join("/"))
                .parse()
                .unwrap(),
        )?),
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    env_logger::init();
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
            .wrap(Logger::default())
            .route("/login", post().to(login))
            .route("/graphql", post().to(graphql))
            .route("/graphiql", get().to(graphiql))
            .route("/{path:.*}", get().to(index))
    })
    .bind(&url)
    .expect("Failed to start Total Recall")
    .run()
    .await
}
