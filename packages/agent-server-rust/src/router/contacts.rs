use axum::{extract::Query, Json};
use serde::Deserialize;

use crate::context::create_context;
use crate::db::get_db;
use crate::ia::types::Contact;
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

pub async fn list_contacts(
    ActiveSession(session): ActiveSession,
    Query(params): Query<ListParams>,
) -> Json<Vec<Contact>> {
    let logged_in_user = match &session.logged_in_user {
        Some(u) => u.clone(),
        None => return Json(Vec::new()),
    };

    let mut keys = {
        let db = get_db();
        get_stored_keys(&db, &session.id, &logged_in_user)
    };

    // Lazy key extraction: if contact.db exists on disk without stored key, re-extract
    if !keys.contains_key("contact.db") {
        let on_disk = list_account_dbs(&logged_in_user);
        if on_disk.iter().any(|name| name == "contact.db") {
            if let Some(pid) = find_wechat_pid_for_user(&session.linux_user) {
                let extracted = extract_keys_async(pid).await;
                if !extracted.is_empty() {
                    let db = get_db();
                    store_keys(&db, &session.id, &logged_in_user, &extracted);
                    keys = get_stored_keys(&db, &session.id, &logged_in_user);
                }
            }
        }
    }

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
    let logged_in_user = match &session.logged_in_user {
        Some(u) => u.clone(),
        None => return Json(Vec::new()),
    };

    let mut keys = {
        let db = get_db();
        get_stored_keys(&db, &session.id, &logged_in_user)
    };

    // Lazy key extraction: if contact.db exists on disk without stored key, re-extract
    if !keys.contains_key("contact.db") {
        let on_disk = list_account_dbs(&logged_in_user);
        if on_disk.iter().any(|name| name == "contact.db") {
            if let Some(pid) = find_wechat_pid_for_user(&session.linux_user) {
                let extracted = extract_keys_async(pid).await;
                if !extracted.is_empty() {
                    let db = get_db();
                    store_keys(&db, &session.id, &logged_in_user, &extracted);
                    keys = get_stored_keys(&db, &session.id, &logged_in_user);
                }
            }
        }
    }

    Json(wechat_contacts::find_contacts(
        &logged_in_user,
        &keys,
        &params.name,
    ))
}

pub async fn current_profile(ActiveSession(session): ActiveSession) -> Json<Option<Contact>> {
    let username = match session.logged_in_user.clone() {
        Some(user) => user,
        None => return Json(None),
    };
    let nick_name = {
        let db = get_db();
        create_context(session, &db).state.main_window.account_name
    };

    Json(nick_name.map(|nick_name| Contact {
        username,
        nick_name,
        remark: None,
        alias: None,
        big_head_url: None,
        small_head_url: None,
        contact_type: "individual".to_string(),
    }))
}
