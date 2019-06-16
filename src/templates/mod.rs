use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index;

pub enum CourtKind {
    Tennis,
}

#[derive(Template)]
#[template(path = "court_overview.html")]
pub struct CourtOverview {
    pub name: String,
    pub occupied: bool,
    pub kind: String,
}

pub struct CourtInfo {
    pub id: u32,
    pub name: String,
    pub occupied: bool,
    pub expected_occupancy: Option<String>,
    pub court_kind: String,
}

#[derive(Template)]
#[template(path = "all_courts.html")]
pub struct AllCourts {
    pub courts: Vec<CourtInfo>,
}
