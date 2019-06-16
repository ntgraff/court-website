use actix_files::NamedFile;
use actix_web::get;
use actix_web::{
    guard, http::StatusCode, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result,
};
use askama::Template;
use log::{debug, info};
use mysql::params;
use std::io::{Read, Write};
use std::path::Path;

mod templates;
use templates::*;

const DATABASE_NAME: &str = "neucourts";

fn courts(pool: web::Data<mysql::Pool>, path: web::Path<(u32,)>) -> Result<HttpResponse> {
    info!("courts path is: {:?}", path);
    match pool
        .prep_exec(
            "SELECT court_id, name, occupied, court_type FROM courts WHERE court_id = :court_id",
            params! {
                "court_id" => path.0
            },
        )
        .map(|result| {
            result
                .map(|x| x.unwrap())
                .map(|row| {
                    let (_, name, occupied, kind) =
                        mysql::from_row::<(u32, String, bool, String)>(row);
                    CourtOverview {
                        name,
                        occupied,
                        kind,
                    }
                })
                .collect::<Vec<_>>()
        }) {
        Ok(res) => {
            if res.is_empty() {
                Ok(HttpResponse::Ok()
                    .content_type("text/html")
                    .status(StatusCode::NOT_FOUND)
                    .body(include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/static/404.html"
                    ))))
            } else {
                let content = &res[0];
                Ok(HttpResponse::Ok()
                    .content_type("text/html")
                    .body(content.render().unwrap()))
            }
        }
        Err(_) => Ok(HttpResponse::Ok()
            .content_type("text/html")
            .status(StatusCode::NOT_FOUND)
            .body(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static/404.html"
            )))),
    }
}

fn courts_index(pool: web::Data<mysql::Pool>) -> HttpResponse {
    let courts = {
        let courts = pool
            .prep_exec(
                "SELECT court_id, name, occupied, expected_occupancy, court_type FROM courts",
                (),
            )
            .unwrap()
            .map(|x| x.unwrap())
            .map(|row| {
                let (id, name, occupied, expected_occupancy, court_kind) =
                    mysql::from_row::<(u32, String, bool, Option<String>, String)>(row);
                CourtInfo {
                    id,
                    name,
                    occupied,
                    expected_occupancy,
                    court_kind,
                }
            })
            .collect::<Vec<_>>();
        AllCourts { courts }
    };
    HttpResponse::Ok()
        .content_type("text/html")
        .body(courts.render().unwrap())
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
    let password = rpassword::prompt_password_stdout("Password: ").expect("not a valid password");

    HttpServer::new(move || {
        let mut opts = mysql::OptsBuilder::new();
        opts.ip_or_hostname(Some("localhost"))
            .user(Some(username.clone()))
            .pass(Some(password.clone()))
            .db_name(Some(DATABASE_NAME));

        let pool = mysql::Pool::new(opts).unwrap();
        pool.get_conn()
            .unwrap()
            .query(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/sql/table_creation.sql"
            )))
            .unwrap();

        App::new()
            .data(pool)
            .wrap(middleware::Logger::default())
            .service(web::resource("/").route(web::get().to(home)))
            .service(web::resource("/courts").route(web::get().to(courts_index)))
            .service(web::resource("/courts/{id}").route(web::get().to(courts)))
            .default_service(
                web::resource("").route(web::get().to(p404)).route(
                    web::route()
                        .guard(guard::Not(guard::Get()))
                        .to(HttpResponse::MethodNotAllowed),
                ),
            )
    })
    .bind("127.0.0.1:8000")?
    .start();

    println!("Starting on 127.0.0.1:8000");
    sys.run()
}
