#[macro_use]
extern crate log;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;

use cid::Cid;
use diesel::{Connection, PgConnection};
use failure::{Error, ResultExt};
use std::convert::TryFrom;
use std::env;

pub mod db;
pub mod model;
pub mod schema;

pub type Result<T> = std::result::Result<T, Error>;

pub fn establish_connection() -> Result<PgConnection> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let conn = PgConnection::establish(&database_url)
        .context(format!("error connecting to {}", database_url))?;

    Ok(conn)
}

pub fn canonicalize_cid_from_str(cid: &str) -> Result<String> {
    let provided_cid = Cid::try_from(cid).context("invalid CID")?;
    Ok(canonicalize_cid(&provided_cid))
}

pub fn canonicalize_cid(c: &Cid) -> String {
    let v1_cid = Cid::new_v1(c.codec(), c.hash().to_owned());
    multibase::encode(multibase::Base::Base32Lower, v1_cid.to_bytes())
}