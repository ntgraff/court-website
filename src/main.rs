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
            .service(web::resource("/court/{id}").route(web::get().to(court_info)))
            .service(web::resource("/reserve/{id}").route(web::post().to(reserve_court)))
            .service(actix_files::Files::new("/static", "./static"))
            .default_service(
                web::resource("")
                    .route(web::get().to(p404))
                    .route(web::post().to(|data: String| info!("{:?}", data)))
                    .route( web::route()
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

fn court_info(pool: web::Data<mysql::Pool>, path: web::Path<(u32,)>) -> Result<HttpResponse> {
    info!("getting info for court {}", path.0);
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
                            let (id, username, start, end, _, _) =  mysql::from_row::<(u32, String, String, String, u32, Option<u32>)>(row);
                            let party = pool
                                .first_exec("CALL reservation_available_party(:rid)", params!("rid" => id))
                                .unwrap()
                                .map(|row| {
                                    let (id, capacity, current) = mysql::from_row::<(u32, u32, u32)>(row);
                                    PartyInfo {
                                        id,
                                        capacity,
                                        current,
                                    }
                                });
                            ReservationInfo { id, username, start, end, party }
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

#[derive(Serialize, Deserialize)]
struct ReserveForm {
    start_date: String,
    start_time: String,
    end_date: String,
    end_time: String,
    username: String,
    password: String,
}

fn reserve_court(
    pool: web::Data<mysql::Pool>,
    content: web::Form<ReserveForm>,
    path: web::Path<(u32,)>,
) -> HttpResponse {
    let court_id = path.0;
    trace!("reserving court with id {}", court_id);
    let start = format!("{} {}", content.start_date.clone(), content.start_time.clone());
    let end = format!("{} {}", content.end_date.clone(), content.end_time.clone());
    let dt_format = r"%Y-%m-%d %H:%i";
    let username = content.username.clone();
    let password = content.password.clone();
    if try_login(&pool, username.clone(), password.clone()) {
        let can_reserve = {
            let row = pool
                .first_exec(
                    "SELECT can_reserve_between(:court_id, STR_TO_DATE(:start, :format), STR_TO_DATE(:end, :format))",
                    params! {
                        "court_id" => &court_id,
                        "start" => &start,
                        "end" => &end,
                        "format" => dt_format,
                    },
                )
                .unwrap()
                .unwrap();
            mysql::from_row::<(bool,)>(row).0
        };
        if can_reserve {
            let _ = pool.prep_exec(
                "CALL add_reservation(:court_id, STR_TO_DATE(:start, :format), STR_TO_DATE(:end, :format), :username)",
                params! {
                    "court_id" => &court_id,
                    "start" => &start,
                    "end" => &end,
                    "username" => &username,
                    "format" => dt_format,
                },
            );
            HttpResponse::Ok().body("reserved!")
        } else {
            HttpResponse::BadRequest().body("already reserved for that period!")
        }
    } else {
        HttpResponse::BadRequest().body("login failed!")
    }
}

// TODO
//   * FIX ALL POSTS
//   * add cookie login?
//   * add create party
//   * __CSS__
//   * some other stuff, need to look at app again
//   * fix login and register page

fn try_login(pool: &web::Data<mysql::Pool>, username: String, password: String) -> bool {
    let row = pool
        .first_exec(
            "SELECT successful_login(:username, :password)",
            params!(username, password),
        )
        .unwrap()
        .unwrap();
    mysql::from_row::<(bool,)>(row).0
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
