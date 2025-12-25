use uzers::{get_user_by_uid, get_current_uid};

const PISCINER_GROUP: &str = "2026_lyon";

pub fn is_authorized() -> bool {
	if let Some(user) = get_user_by_uid(get_current_uid()) {
		if user.groups().unwrap_or(vec![]).into_iter().any(|g| g.name() == PISCINER_GROUP) {
			return false;
		}
	}
	true
}
