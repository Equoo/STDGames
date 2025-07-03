use users::{get_user_by_uid, get_current_uid};

const PISCINER_GROUP: &str = "2025_lyon";

pub fn is_authorized() -> Option<String> {
    if let Some(user) = get_user_by_uid(get_current_uid()) {
        if user.groups().unwrap_or(vec![]).into_iter().any(|g| g.name() == PISCINER_GROUP) {
            return Some("les piscineux n'ont pas accès a stdgames".to_string());
        }
    }
    None
}
