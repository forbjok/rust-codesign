mod error;
mod signtool;

pub use error::*;
pub use signtool::*;

pub struct SignParams {
    pub digest_algorithm: String,
    pub certificate_thumbprint: String,
    pub timestamp_url: Option<String>,
}
