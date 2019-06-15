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
