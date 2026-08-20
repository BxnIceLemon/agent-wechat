use super::wechat_db::{get_db_path, query_wechat_db};
use crate::ia::types::Contact;
use serde_json::Value;
use std::collections::HashMap;

/// Known system/internal usernames to exclude from contact listings.
const SYSTEM_USERNAMES: &[&str] = &[
    "qmessage",
    "floatbottle",
    "medianote",
    "notifymessage",
    "weixin",
    "fmessage",
    "filehelper",
    "newsapp",
    "tmessage",
    "mphelper",
    "qqmail",
    "weixingongzhong",
    "qqsafe",
    "exmail_tool",
    "lbsapp",
    "pc_qq",
];

fn classify_contact(username: &str, local_type: i64) -> &'static str {
    if username.ends_with("@chatroom") {
        "chatroom"
    } else if username.starts_with("gh_") {
        "official"
    } else if username.contains("@openim") || local_type == 5 {
        "openim"
    } else {
        "individual"
    }
}

fn is_system_account(username: &str) -> bool {
    SYSTEM_USERNAMES.contains(&username)
}

fn contact_from_row(row: &Value) -> Option<Contact> {
    let username = row.get("username")?.as_str()?;
    if is_system_account(username) {
        return None;
    }

    let local_type = row
        .get("local_type")
        .and_then(|value| value.as_i64())
        .unwrap_or(3);
    let text = |name| {
        row.get(name)
            .and_then(|value| value.as_str())
            .filter(|value| !value.is_empty())
            .map(String::from)
    };
    Some(Contact {
        username: username.to_string(),
        nick_name: row
            .get("nick_name")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string(),
        remark: text("remark"),
        alias: text("alias"),
        big_head_url: text("big_head_url"),
        small_head_url: text("small_head_url"),
        contact_type: classify_contact(username, local_type).to_string(),
    })
}

/// List contacts from contact.db.
/// Queries the contact table directly (not session.db), returning all stored contacts.
pub fn list_contacts(
    account_dir: &str,
    keys: &HashMap<String, String>,
    limit: i64,
    offset: i64,
) -> Vec<Contact> {
    let contact_key = match keys.get("contact.db") {
        Some(k) => k,
        None => return Vec::new(),
    };

    let contact_db = get_db_path(account_dir, "contact.db");

    // local_type: 0=system notifications, 1=contacts+official, 2=chatrooms, 3=contacts, 5=openim
    // Include 1, 3, 5 (skip 0=system notifications, 2=chatrooms)
    let rows = query_wechat_db(
        &contact_db,
        contact_key,
        &format!(
            "SELECT username, nick_name, remark, alias, big_head_url, small_head_url, local_type
             FROM contact
             WHERE local_type IN (1, 3, 5)
               AND username NOT LIKE '%@chatroom'
             ORDER BY remark != '' DESC, nick_name COLLATE NOCASE ASC
             LIMIT {limit} OFFSET {offset};"
        ),
    );

    rows.iter().filter_map(contact_from_row).collect()
}

/// Search contacts by name (partial match on nick_name, remark, alias, or username).
pub fn find_contacts(
    account_dir: &str,
    keys: &HashMap<String, String>,
    query: &str,
) -> Vec<Contact> {
    let contact_key = match keys.get("contact.db") {
        Some(k) => k,
        None => return Vec::new(),
    };

    let contact_db = get_db_path(account_dir, "contact.db");
    let escaped = query.replace('\'', "''");

    let rows = query_wechat_db(
        &contact_db,
        contact_key,
        &format!(
            "SELECT username, nick_name, remark, alias, big_head_url, small_head_url, local_type
             FROM contact
             WHERE local_type IN (1, 3, 5)
               AND username NOT LIKE '%@chatroom'
               AND (nick_name LIKE '%{escaped}%'
                    OR remark LIKE '%{escaped}%'
                    OR alias LIKE '%{escaped}%'
                    OR username LIKE '%{escaped}%')
             ORDER BY remark != '' DESC, nick_name COLLATE NOCASE ASC
             LIMIT 50;"
        ),
    );

    rows.iter().filter_map(contact_from_row).collect()
}

/// Read the profile row belonging to the currently logged-in account.
pub fn get_profile(
    account_dir: &str,
    keys: &HashMap<String, String>,
    logged_in_user: &str,
) -> Option<Contact> {
    let contact_key = keys.get("contact.db")?;
    let escaped = logged_in_user.replace('\'', "''");
    let rows = query_wechat_db(
        &get_db_path(account_dir, "contact.db"),
        contact_key,
        &format!(
            "SELECT username, nick_name, remark, alias, big_head_url, small_head_url, local_type
             FROM contact
             WHERE local_type IN (1, 3, 5)
               AND (username = '{escaped}'
                    OR substr('{escaped}', 1, length(username) + 1) = username || '_')
             ORDER BY length(username) DESC
             LIMIT 1;"
        ),
    );
    rows.first().and_then(contact_from_row)
}
