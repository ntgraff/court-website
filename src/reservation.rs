use crate::templates::*;
use actix_identity::Identity;
use actix_web::{
    http::{self, StatusCode},
    web, HttpResponse, Result,
};
use askama::Template;
use log::{debug, error, info, trace};
use mysql::params;
use serde::{Deserialize, Serialize};

pub fn reservations(pool: web::Data<mysql::Pool>, id: Identity) -> HttpResponse {
    match id.identity() {
        Some(username) => {
            let reservations = pool
                .prep_exec(
                    "SELECT r.reservation_id, r.court_id, name \
                    FROM reservations r JOIN courts c ON r.court_id = c.court_id \
                    WHERE username = :uid AND end_time > NOW()",
                    params!("uid" => username),
                )
                .unwrap()
                .map(|row| {
                    let row = row.unwrap();
                    let (id, court_id, court_name) = mysql::from_row::<(u32, u32, String)>(row);
                    RegistrationOverview { id, court_id, court_name, }
                })
                .collect::<Vec<_>>();

            HttpResponse::Ok()
                .content_type("text/html")
                .body(AllReservations { signed_in: true, reservations }.render().unwrap() )
        }
        None => HttpResponse::BadRequest().body("Not logged in!"),
    }
}

#[derive(Deserialize)]
pub struct ReservationRemove {
    to_remove: String,
}

pub fn remove_reservation(pool: web::Data<mysql::Pool>, id: Identity, content: web::Form<ReservationRemove>) -> HttpResponse {
    match id.identity() {
        Some(username) => {
            dbg!(pool.first_exec(
                "DELETE FROM reservations WHERE reservation_id = :rid",
                params!("rid" => content.to_remove.parse::<u32>().unwrap()),
            ));
            HttpResponse::Found().header("location", "/reservations").finish()
        }
        None => HttpResponse::BadRequest().body("not logged in"),
    }
}
