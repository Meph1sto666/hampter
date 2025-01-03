use std::collections::HashMap;

use crate::auth::AuthorizedClient;
use chrono;
use getters2::Getters;
use serde_json::json;

use super::error::HampterError;

#[derive(serde::Deserialize, serde::Serialize, Getters)]
pub struct Tag {
	id: u32,
	#[serde(deserialize_with = "deserialize_datetime")]
	created_at: chrono::DateTime<chrono::Utc>,
	name: String,
	slug: String,
	description: String,
}
fn deserialize_datetime<'de, D>(deserializer: D) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
	D: serde::Deserializer<'de>,
{
	let s: &str = serde::Deserialize::deserialize(deserializer)?;

	match chrono::DateTime::parse_from_rfc3339(s) {
		Ok(dt) => Ok(dt.with_timezone(&chrono::Utc)),
		Err(_) => match chrono::DateTime::parse_from_rfc3339(&format!("{}:00", s)) {
			Ok(dt) => Ok(dt.with_timezone(&chrono::Utc)),
			Err(_) => Ok(chrono::Utc::now()),
		},
	}
}

impl Tag {
	/**
	 * Request the list of "official" tags introduced by JanitorAI
	 */
	pub async fn get_tag_list(client: &AuthorizedClient) -> Result<Vec<Tag>, HampterError> {
		Ok(client
			.client()
			.get("https://janitorai.com/hampter/tags")
			.send()
			.await?
			.error_for_status()?
			.json::<Vec<Tag>>()
			.await?
		)
	}

	/**
	 * Returns the list of custom tags the client is following
	 */
	pub async fn get_following_tags(client: &AuthorizedClient) -> Result<Vec<String>, HampterError> {
		/**
		 * These are custom tags
		 */
		#[derive(serde::Deserialize)]
		struct FollowTags {
			following_tags: Vec<String>,
		}

		Ok(client
			.client()
			.get("https://janitorai.com/hampter/following/tags")
			.send()
			.await?
			.error_for_status()?
			.json::<FollowTags>()
			.await?
			.following_tags
		)
	}

	/**
	 * Update the list of the tags the client is following
	 * This overwrites the previous list!
	 */
	pub async fn follow_tags(tags: Vec<&str>, client: &AuthorizedClient) -> Result<(), HampterError> {
		client
			.client()
			.post("https://janitorai.com/hampter/following/tags")
			.json(&json!({
				"custom_tags": tags
			}))
			.send()
			.await?
			.error_for_status()?;
		Ok(())
	}

	/**
	 * Query the server for custom tags
	 * Search query `prefix` has to be at least 3 letters long for a server response.
	 * THe server response consists of an array with zero to five custom tags.
	 */
	pub async fn get_tag_suggestion(
		prefix: &str,
		client: &AuthorizedClient,
	) -> Result<Vec<String>, HampterError> {
		if prefix.len() < 3 {
			return Ok(vec![]);
		}
		Ok(client
			.client()
			.get(format!(
				"https://janitorai.com/hampter/characters/tags/suggest?prefix={}",
				prefix
			))
			.send()
			.await?
			.error_for_status()?
			.json::<HashMap<String, Vec<String>>>()
			.await?
			.get("suggestions")
			.get_or_insert(&vec![])
			.to_owned())
	}
}
