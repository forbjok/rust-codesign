extern crate bitness;
#[macro_use] extern crate failure;
#[macro_use] extern crate log;
extern crate winreg;

mod result;
mod signtool;

pub use result::*;
pub use signtool::*;

pub struct SignParams {
    pub digest_algorithm: String,
    pub certificate_thumbprint: String,
    pub timestamp_url: Option<String>,
}
