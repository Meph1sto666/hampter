use super::error::HampterError;
use crate::auth::AuthorizedClient;
use chrono;
use getters2::Getters;
use serde;

#[derive(serde::Deserialize, serde::Serialize, Getters)]
pub struct UserProfile {
	name: String,
	avatar: String,
	user_name: Option<String>,
	is_verified: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Getters)]
pub struct Review {
	id: String,
	user_id: String,
	character_id: String,
	created_at: chrono::DateTime<chrono::Utc>,
	content: Option<String>,
	is_like: bool,
	like_count: u32,
	comment_count: u32,
	user_profiles: UserProfile,
	moderator: bool,
}

impl Review {
	/**
	 * Request the reviews of a bot
	 */
	pub async fn get(
		character_id: &str,
		client: &AuthorizedClient,
	) -> Result<Vec<Review>, HampterError> {
		Ok(client
			.client()
			.get(format!(
				"https://janitorai.com/hampter/reviews/{}",
				character_id
			))
			.send()
			.await?
			.error_for_status()?
			.json::<Vec<Review>>()
			.await?)
	}
}
