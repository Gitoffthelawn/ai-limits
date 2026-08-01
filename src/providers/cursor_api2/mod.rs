mod fetch;
mod helpers;
mod parse;
mod project;

use std::io;

use crate::types::SourceData;

pub use project::build_source_data;

use fetch::fetch_usage_response;
use project::access_denied;

pub fn collect_usage() -> io::Result<SourceData> {
    match fetch_usage_response()? {
        Ok(response) => Ok(build_source_data(&response)),
        Err(denied) => Ok(access_denied(denied.message, denied.raw)),
    }
}
