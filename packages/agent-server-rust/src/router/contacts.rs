use axum::{extract::Query, Json};
use serde::Deserialize;
use std::collections::HashMap;

use crate::db::get_db;
use crate::ia::types::{Contact, Session};
use crate::router::session::ActiveSession;
use crate::tools::wechat_contacts;
use crate::tools::wechat_db::{find_wechat_pid_for_user, list_account_dbs};
use crate::tools::wechat_keys::{extract_keys_async, get_stored_keys, store_keys};

#[derive(Deserialize)]
pub struct ListParams {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    200
}

async fn account_keys(session: &Session) -> Option<(String, HashMap<String, String>)> {
    let logged_in_user = session.logged_in_user.clone()?;
    let mut keys = {
        let db = get_db();
        get_stored_keys(&db, &session.id, &logged_in_user)
    };

    if !keys.contains_key("contact.db")
        && list_account_dbs(&logged_in_user)
            .iter()
            .any(|name| name == "contact.db")
    {
        if let Some(pid) = find_wechat_pid_for_user(&session.linux_user) {
            let extracted = extract_keys_async(pid).await;
            if !extracted.is_empty() {
                let db = get_db();
                store_keys(&db, &session.id, &logged_in_user, &extracted);
                keys = get_stored_keys(&db, &session.id, &logged_in_user);
            }
        }
    }

    Some((logged_in_user, keys))
}

pub async fn list_contacts(
    ActiveSession(session): ActiveSession,
    Query(params): Query<ListParams>,
) -> Json<Vec<Contact>> {
    let (logged_in_user, keys) = match account_keys(&session).await {
        Some(account) => account,
        None => return Json(Vec::new()),
    };

    if !keys.contains_key("contact.db") {
        return Json(Vec::new());
    }

    Json(wechat_contacts::list_contacts(
        &logged_in_user,
        &keys,
        params.limit,
        params.offset,
    ))
}

#[derive(Deserialize)]
pub struct FindParams {
    name: String,
}

pub async fn find_contacts(
    ActiveSession(session): ActiveSession,
    Query(params): Query<FindParams>,
) -> Json<Vec<Contact>> {
    let (logged_in_user, keys) = match account_keys(&session).await {
        Some(account) => account,
        None => return Json(Vec::new()),
    };

    Json(wechat_contacts::find_contacts(
        &logged_in_user,
        &keys,
        &params.name,
    ))
}

pub async fn current_profile(ActiveSession(session): ActiveSession) -> Json<Option<Contact>> {
    let (logged_in_user, keys) = match account_keys(&session).await {
        Some(account) => account,
        None => return Json(None),
    };

    Json(wechat_contacts::get_profile(
        &logged_in_user,
        &keys,
        &logged_in_user,
    ))
}
