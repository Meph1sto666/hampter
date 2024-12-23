use std::collections::HashMap;

use crate::auth::AuthorizedClient;
use chrono;
use getters2::Getters;
use serde_json::json;

#[derive(serde::Deserialize, serde::Serialize, Getters)]
pub struct Tag {
	id: u32,
	#[serde(deserialize_with="deserialize_datetime")]
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
		Err(_) => {
			match chrono::DateTime::parse_from_rfc3339(&format!("{}:00", s)) {
				Ok(dt) => Ok(dt.with_timezone(&chrono::Utc)),
				Err(_) => Ok(chrono::Utc::now())
			}
		}
	}
}

impl Tag {
	pub async fn get_tag_list(client: &AuthorizedClient) -> Vec<Tag> {
		client
			.client()
			.get("https://janitorai.com/hampter/tags")
			.send()
			.await
			.expect("Failed to send request")
			.error_for_status()
			.expect("Invalid response")
			.json::<Vec<Tag>>()
			.await
			.expect("Failed to parse tag list")
	}

	pub async fn get_following_tags(client: &AuthorizedClient) -> Vec<String> {
		/**
		 * These are custom tags
		 */
		#[derive(serde::Deserialize)]
		struct FollowTags {
			following_tags: Vec<String>,
		}

		client
			.client()
			.get("https://janitorai.com/hampter/following/tags")
			.send()
			.await
			.expect("Failed to send request")
			.error_for_status()
			.expect("Invalid response")
			.json::<FollowTags>()
			.await
			.expect("Failed to parse followed tags")
			.following_tags
	}

	pub async fn follow_tags(tags: Vec<&str>, client: &AuthorizedClient) {
		client
			.client()
			.post("https://janitorai.com/hampter/following/tags")
			.json(&json!({
				"custom_tags": tags
			}))
			.send()
			.await
			.expect("Failed to post follow tags")
			.error_for_status()
			.expect("Invalid response");
	}
	
	/// Search query `prefix` has to be at least 3 letters long for a server response.
	pub async fn get_tag_suggestion(prefix: &str, client: &AuthorizedClient) -> Vec<String> {
		if prefix.len() < 3 {return vec![];}

		client
			.client()
			.get(format!("https://janitorai.com/hampter/characters/tags/suggest?prefix={}", prefix))
			.send()
			.await
			.expect("Failed to send request")
			.error_for_status()
			.expect("Invalid response")
			.json::<HashMap<String, Vec<String>>>()
			.await
			.expect("Failed to parse followed tags").get("suggestions").expect("Malformed response, missing 'suggestions' key").to_owned()
	}
}
