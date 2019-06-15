use log::info;
use actix_files::NamedFile;
use actix_web::get;
use actix_web::{
    guard, http::StatusCode, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result,
};
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
struct Index;

enum CourtKind {
    Tennis,
}

#[derive(Template)]
#[template(path = "court_overview.html")]
struct CourtOverview {
    name: String,
    occupied: bool,
    kind: String,
}

fn courts(pool: web::Data<mysql::Pool>, path: web::Path<(String,)>) -> Result<HttpResponse> {
    info!("courts path is: {:?}", path);
    let overviews = pool
        .prep_exec("SELECT name, occupied, kind FROM courts", ())
        .map(|result| {
            result
                .map(|x| x.unwrap())
                .map(|row| {
                    let (name, occupied, kind): (String, _, String) = mysql::from_row(row);
                    CourtOverview {
                        name,
                        occupied,
                        kind,
                    }
                })
                .take(1)
                .collect::<Vec<_>>()
        })
        .unwrap();
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(overviews[0].render().unwrap()))
}

fn home() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(Index.render().unwrap())
}

fn p404() -> Result<NamedFile> {
    Ok(NamedFile::open("static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
}

fn main() -> std::io::Result<()> {
    env_logger::init();
    let sys = actix_rt::System::new("database-project");

    HttpServer::new(|| {
        let mut opts = mysql::OptsBuilder::new();
        opts.ip_or_hostname(Some("localhost"))
            .user(Some("dbproj"))
            .pass(Some("justanex"))
            .db_name(Some("courtboi"));

        let pool = mysql::Pool::new(opts).unwrap();
        pool.prep_exec(
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/sql/table_creation.sql"
            )),
            (),
        )
        .unwrap();

        App::new()
            .data(pool)
            .wrap(middleware::Logger::default())
            .service(web::resource("/").route(web::get().to(home)))
            .service(web::resource("/courts/{id}").route(web::get().to(courts)))
            .default_service(
                web::resource("").route(web::get().to(p404)).route(
                    web::route()
                        .guard(guard::Not(guard::Get()))
                        .to(|| HttpResponse::MethodNotAllowed()),
                ),
            )
    })
    .bind("127.0.0.1:8000")?
    .start();

    println!("Starting on 127.0.0.1:8000");
    sys.run()
}
