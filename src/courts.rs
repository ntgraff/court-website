use crate::templates::*;
use actix_identity::Identity;
use actix_web::{
    http::{self, StatusCode},
    web, HttpResponse, Result,
};
use askama::Template;
use log::{debug, error, info, trace};
use mysql::params;

pub fn court_info(
    pool: web::Data<mysql::Pool>,
    uid: Identity,
    path: web::Path<(u32,)>,
) -> Result<HttpResponse> {
    match court_data(&*pool, path.0) {
        Ok(court) => {
            let reservations = pool
                .prep_exec("Call court_reservations(:cid)", params!("cid" => court.id))
                .unwrap()
                .map(|row| {
                    let row = row.unwrap();
                    let (id, username, start, end, _, _) =
                        mysql::from_row::<(u32, String, String, String, u32, Option<u32>)>(row);
                    let party = pool
                        .first_exec(
                            "CALL reservation_available_party(:rid)",
                            params!("rid" => id),
                        )
                        .unwrap()
                        .map(|row| {
                            let (id, capacity, current) = mysql::from_row::<(u32, u32, u32)>(row);
                            PartyInfo {
                                id,
                                capacity,
                                current,
                            }
                        });
                    ReservationInfo {
                        id,
                        username,
                        start,
                        end,
                        party,
                    }
                })
                .collect::<Vec<_>>();
            let court_info = CourtInfo {
                id: court.id,
                name: court.name,
                occupied: court.occupied,
                kinds: court.kinds,
                reservations,
                signed_in: uid.identity().is_some(),
            };
            Ok(HttpResponse::Ok()
                .content_type("text/html")
                .body(court_info.render().unwrap()))
        }
        Err(_) => Ok(HttpResponse::BadRequest()
            .content_type("text/html")
            .status(StatusCode::NOT_FOUND)
            .body(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/static/404.html"
            )))),
    }
}

pub fn courts_index(pool: web::Data<mysql::Pool>) -> HttpResponse {
    let courts = {
        let courts = pool
            .prep_exec("SELECT court_id FROM courts", ())
            .unwrap()
            .map(Result::unwrap)
            .map(|row| {
                let id = mysql::from_row::<(u32,)>(row).0;
                let CourtData {
                    id,
                    name,
                    kinds,
                    occupied,
                } = court_data(&*pool, id).unwrap();
                CourtOverview {
                    id,
                    name,
                    kinds,
                    occupied,
                }
            })
            .collect::<Vec<_>>();
        AllCourts { courts }
    };
    HttpResponse::Ok()
        .content_type("text/html")
        .body(courts.render().unwrap())
}

struct CourtData {
    id: u32,
    name: String,
    occupied: bool,
    kinds: Vec<(String, String)>,
}

// could be done in mysql, but already had all the code here among the functions, would reimplement.
fn court_data(pool: &mysql::Pool, cid: u32) -> mysql::Result<CourtData> {
    info!("getting info for court {}", cid);
    match pool.first_exec(
        "SELECT court_id, name, is_occupied(court_id) FROM courts WHERE court_id = :court_id",
        params! {
            "court_id" => cid
        },
    )? {
        Some(row) => {
            let (id, name, occupied) = mysql::from_row::<(u32, String, bool)>(row);
            let kinds = pool
                .prep_exec("CALL court_types(:cid)", params!("cid" => id))?
                .map(|row| {
                    let row = row.unwrap();
                    mysql::from_row::<(String, String)>(row)
                })
                .collect::<Vec<_>>();
            Ok(CourtData {
                id,
                name,
                occupied,
                kinds,
            })
        }
        None => Err(mysql::Error::MySqlError(mysql::MySqlError {
            state: "".to_owned(),
            message: "".to_owned(),
            code: 0,
        })),
    }
}
