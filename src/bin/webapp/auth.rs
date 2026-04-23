use axum::http::{header, HeaderMap, HeaderValue};

use crate::webapp::state::SecurityProfile;

const COOKIE_NAME: &str = "acme_session";

#[derive(Debug, Clone)]
pub struct UserSession {
    pub username: String,
    pub profile: SecurityProfile,
}

pub fn session_from_headers(headers: &HeaderMap) -> Option<UserSession> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;

    for pair in cookie_header.split(';') {
        let trimmed = pair.trim();
        let (name, value) = trimmed.split_once('=')?;
        if name != COOKIE_NAME {
            continue;
        }
        let (username, profile) = value.split_once('|')?;
        return Some(UserSession {
            username: username.to_string(),
            profile: SecurityProfile::from_form_value(profile).ok()?,
        });
    }

    None
}

pub fn set_session_cookie(headers: &mut HeaderMap, session: &UserSession) -> Result<(), axum::http::header::InvalidHeaderValue> {
    let cookie = format!(
        "{}={}|{}; Path=/; HttpOnly; SameSite=Lax",
        COOKIE_NAME,
        session.username,
        session.profile.as_cookie_value()
    );
    let value = HeaderValue::from_str(&cookie)?;
    headers.append(header::SET_COOKIE, value);
    Ok(())
}

pub fn clear_session_cookie(headers: &mut HeaderMap) {
    headers.append(
        header::SET_COOKIE,
        HeaderValue::from_static("acme_session=deleted; Path=/; Max-Age=0; HttpOnly; SameSite=Lax"),
    );
}
