use actix_files::NamedFile;
use actix_web::get;
use actix_web::{
    guard, http::StatusCode, middleware, middleware::identity::Identity, web, App, HttpRequest,
    HttpResponse, HttpServer, Result,
};
use askama::Template;
use log::{debug, info, trace};
use mysql::params;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

mod templates;
use templates::*;

const DATABASE_NAME: &str = "neucourts";

fn main() -> std::io::Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("data_backend", log::LevelFilter::max())
        .filter_module("actix_web", log::LevelFilter::Info)
        .init();

    let sys = actix_rt::System::new("database-project");

    let username = {
        let mut username = String::new();
        print!("MariaDB/MySQL Username: ");
        let _ = std::io::stdout().flush();
        let _ = std::io::stdin().read_line(&mut username);
        username.trim().to_owned()
    };
    let password = rpassword::prompt_password_stdout("MariaDB/MySQL Password: ")
        .expect("not a valid password");

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
            .service(
                web::resource("/login")
                    .route(web::get().to(plogin))
                    .route(web::post().to(login_post)),
            )
            .service(
                web::resource("/register")
                    .route(web::get().to(p404))
                    .route(web::post().to(register_post)),
            )
            .service(web::resource("/courts").route(web::get().to(courts_index)))
            .service(web::resource("/courts/{id}").route(web::get().to(courts)))
            .service(actix_files::Files::new("/static", "./static"))
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

fn courts(pool: web::Data<mysql::Pool>, path: web::Path<(u32,)>) -> Result<HttpResponse> {
    info!("courts path is: {:?}", path);
    match pool
        .prep_exec(
            "SELECT court_id, name, is_occupied(court_id), court_type FROM courts WHERE court_id = :court_id",
            params! {
                "court_id" => path.0
            },
        )
        .map(|result| {
            result
                .map(|x| x.unwrap())
                .map(|row| {
                    let (id, name, occupied, kind) = mysql::from_row::<(u32, String, bool, String)>(row);
                    let reservations = pool
                        .prep_exec("CALL court_reservations(:cid)", params!("cid" => id))
                        .unwrap()
                        .map(Result::unwrap)
                        .map(|row| {
                            let (id, username, start, end, _) =  mysql::from_row::<(u32, String, String, String, u32)>(row);
                            let r = ReservationInfo { id, username, start, end };
                            info!("{:?}", r);
                            r
                        })
                        .collect::<Vec<_>>();
                    CourtInfo {
                        id,
                        name,
                        occupied,
                        reservations,
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
                "SELECT court_id, name, is_occupied(court_id), court_type FROM courts",
                (),
            )
            .unwrap()
            .map(|x| x.unwrap())
            .map(|row| {
                let (id, name, occupied, kind) =
                    mysql::from_row::<(u32, String, bool, String)>(row);
                CourtOverview {
                    id,
                    name,
                    occupied,
                    kind,
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

fn plogin() -> Result<NamedFile> {
    Ok(NamedFile::open("static/login.html")?.set_status_code(StatusCode::OK))
}

// TODO
//   * add function to the reservation fields on the page.
//   * __CSS__
//   * some other stuff, need to look at app again
//   * fix login and register page

fn try_login(
    pool: web::Data<mysql::Pool>,
    username: String,
    password: String,
) -> Result<(), &'static str> {
    let stored_pass = pool
        .prep_exec(
            "SELECT password FROM users WHERE username = :username",
            params! {
                "username" => &username
            },
        )
        .map(|result| {
            result
                .map(|x| x.unwrap())
                .map(mysql::from_row::<(String,)>)
                .take(1)
                .collect::<Vec<_>>()
        });
    match stored_pass {
        Ok(ref pass) if !pass.is_empty() && pass[0].0 == password => Ok(()),
        Ok(ref pass) if pass.is_empty() => Err("incorrect username"),
        _ => Err("incorrect password"),
    }
}

#[derive(Serialize, Deserialize)]
struct Login {
    username: String,
    password: String,
}

fn login_post(
    pool: web::Data<mysql::Pool>,
    content: web::Form<Login>,
    id: Identity,
) -> HttpResponse {
    trace!("login request");
    let stored_pass = pool
        .prep_exec(
            "SELECT password FROM users WHERE username = :username",
            params! {"username" => &content.username},
        )
        .map(|result| {
            result
                .map(|x| x.unwrap())
                .map(mysql::from_row::<(String,)>)
                .take(1)
                .collect::<Vec<_>>()
                .clone()
        });

    match stored_pass {
        Ok(ref pass) if !pass.is_empty() && pass[0].0 == content.password => {
            id.remember(content.username.clone());
            HttpResponse::Found()
                .header(actix_web::http::header::LOCATION, "/")
                .finish()
        }
        _ => HttpResponse::UnprocessableEntity().body("not a valid login"),
    }
}

#[derive(Serialize, Deserialize)]
struct Register {
    username: String,
    password1: String,
    password2: String,
}

fn register_post(pool: web::Data<mysql::Pool>, content: web::Form<Register>) -> HttpResponse {
    trace!("register request");
    if content.password1 == content.password2 {
        let mut transaction = pool.start_transaction(false, None, None).unwrap();
        let already_exists = transaction
            .prep_exec(
                "SELECT COUNT(*) FROM users WHERE username = :username",
                params! {"username" => &content.username},
            )
            .unwrap()
            .map(|x| x.unwrap())
            .map(mysql::from_row::<(u32,)>)
            .take(1)
            .collect::<Vec<_>>()[0]
            .0
            > 0;

        if already_exists {
            HttpResponse::BadRequest().body("User already exists!")
        } else {
            let _ = transaction.prep_exec(
                "INSERT INTO users (username, password) VALUES (:username, :password)",
                params! {
                    "username" => &content.username,
                    "password" => &content.password1
                },
            );
            let _ = transaction.commit();
            HttpResponse::Found()
                .header(actix_web::http::header::LOCATION, "/")
                .finish()
                .into_body()
        }
    } else {
        HttpResponse::BadRequest().body("Passwords do not match")
    }
}
