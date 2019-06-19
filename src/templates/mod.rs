use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index {
    pub signed_in: bool,
}

#[derive(Debug)]
pub struct PartyInfo {
    pub id: u32,
    pub capacity: u32,
    pub current: u32,
}

#[derive(Debug)]
pub struct ReservationInfo {
    pub id: u32,
    pub username: String,
    pub start: String,
    pub end: String,
    pub party: Option<PartyInfo>,
}

#[derive(Template)]
#[template(path = "court_info.html")]
pub struct CourtInfo {
    pub id: u32,
    pub name: String,
    pub occupied: bool,
    pub reservations: Vec<ReservationInfo>,
    pub kinds: Vec<(String,String)>,
    pub signed_in: bool,
}

pub struct CourtOverview {
    pub id: u32,
    pub name: String,
    pub occupied: bool,
}

#[derive(Template)]
#[template(path = "all_courts.html")]
pub struct AllCourts {
    pub courts: Vec<CourtOverview>,
}

