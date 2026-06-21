#[derive(Debug, Default)]
pub struct Options {
    pub display_progress: bool,
    pub request_builder: Option<reqwest::blocking::ClientBuilder>,
}
