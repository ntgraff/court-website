use actix_files::NamedFile;
use actix_web::get;
use actix_web::{
    guard, http::StatusCode, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result,
};
use askama::Template;
use log::{info, debug};
use std::io::{Read, Write};

mod templates;
use templates::*;

const DATABASE_NAME: &str = "neucourts";

fn courts(pool: web::Data<mysql::Pool>, path: web::Path<(String,)>) -> Result<HttpResponse> {
    info!("courts path is: {:?}", path);
    let overviews = pool
        .prep_exec("SELECT name, kind FROM courts", ())
        .map(|result| {
            debug!("{:?}", result);
            result
                .map(|x| x.unwrap())
                .map(|row| {
                    let (name, kind) = mysql::from_row::<(String, String)>(row);
                    CourtOverview {
                        name,
                        occupied: true,
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
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("data-backend", log::LevelFilter::Debug)
        .filter_module("actix_web", log::LevelFilter::Info)
        .init();

    let sys = actix_rt::System::new("database-project");

    let username = {
        let mut username = String::new();
        print!("Username: ");
        let _ = std::io::stdout().flush();
        let _ = std::io::stdin().read_line(&mut username);
        username.trim().to_owned()
    };
    let password =
        rpassword::prompt_password_stdout("Password: ").expect("not a valid password");

    HttpServer::new(move || {
        let mut opts = mysql::OptsBuilder::new();
        opts.ip_or_hostname(Some("localhost"))
            .user(Some(username.clone()))
            .pass(Some(password.clone()))
            .db_name(Some(DATABASE_NAME));

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
