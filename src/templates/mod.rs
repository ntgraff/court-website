use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index;


#[derive(Debug)]
pub struct ReservationInfo {
    pub id: u32,
    pub username: String,
    pub start: String,
    pub end: String,
}

#[derive(Template)]
#[template(path = "court_info.html")]
pub struct CourtInfo {
    pub id: u32,
    pub name: String,
    pub occupied: bool,
    pub reservations: Vec<ReservationInfo>,
    pub kind: String,
}

pub struct CourtOverview {
    pub id: u32,
    pub name: String,
    pub occupied: bool,
    pub kind: String,
}

#[derive(Template)]
#[template(path = "all_courts.html")]
pub struct AllCourts {
    pub courts: Vec<CourtOverview>,
}

