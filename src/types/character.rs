use crate::auth::AuthorizedClient;

use super::{misc, tag::Tag};
use getters2::Getters;
use serde;

#[derive(serde::Deserialize, serde::Serialize, Getters)]
pub struct CharacterStats {
	chat: u64,
	message: u64,
}

#[derive(serde::Deserialize, serde::Serialize, Getters)]
pub struct TokenStats {
	scenario_tokens: u32,
	personality_tokens: u32,
	first_message_tokens: u32,
	example_dialog_tokens: u32,
	total_tokens: u32,
}

#[derive(serde::Deserialize, serde::Serialize, Getters)]
pub struct Character {
	id: String,
	name: String,
	avatar: String,
	description: String,
	chat_name: Option<String>,
	created_at: String,
	updated_at: String,
	first_published_at: String,
	is_public: bool,
	is_force_remove: bool,
	#[serde(default)]
	is_deleted: bool,
	showdefinition: Option<bool>,
	#[serde(rename = "showDefinitionOverride")]
	show_definition_override: Option<bool>,
	allow_proxy: Option<bool>,
	is_nsfw: bool,
	creator_id: String,
	creator_name: String,
	creator_verified: bool,
	custom_tags: Option<Vec<String>>,
	soundcloud_track_id: Option<String>,
	token_counts: Option<TokenStats>,
	is_image_nsfw: Option<bool>,
	tags: Vec<Tag>,
	stats: CharacterStats,
}

#[derive(serde::Deserialize, serde::Serialize, Getters)]
pub struct QueryResponse {
	data: Vec<Character>,
	total: u64,
	size: u8,
	#[serde(deserialize_with = "misc::u64_from_string")]
	page: u64, // original is string
	top_custom_tags: Vec<String>,
}

pub enum SortMode {
	Popular,
	Latest,
	Trending,
	Trending24,
	Relevance,
}
impl ToString for SortMode {
	fn to_string(&self) -> String {
		match self {
			Self::Popular => "popular".to_string(),
			Self::Latest => "latest".to_string(),
			Self::Trending => "trending".to_string(),
			Self::Trending24 => "trending24".to_string(),
			Self::Relevance => "relevance".to_string(),
		}
	}
}

impl Character {
	pub async fn get(id: &str, client: &AuthorizedClient) -> Character {
		client
			.client()
			.get(format!("https://janitorai.com/hampter/characters/{}", id))
			.send()
			.await
			.expect("Failed to send request")
			.error_for_status()
			.expect("Invalid response")
			.json::<Character>()
			.await
			.expect("Failed to format response")
	}
	pub async fn query(
		client: &AuthorizedClient,
		page: u32,
		nsfw: bool,
		search: Option<&str>,
		mut sort: Option<SortMode>,
		mut tag_ids: Option<Vec<u32>>,
		mut custom_tags: Option<Vec<&str>>,
	) -> QueryResponse {
		let mut query_string: String = format!(
			"https://janitorai.com/hampter/characters?page={page}&mode={mode}&sort={sort}",
			sort = sort.get_or_insert(SortMode::Popular).to_string(),
			mode = if nsfw { "all" } else { "sfw" }
		);
		for t in tag_ids.get_or_insert(vec![]) {
			query_string.push_str(format!("&tag_id[]={}", t).as_str());
		}
		for t in custom_tags.get_or_insert(vec![]) {
			query_string.push_str(format!("&custom_tags[]={}", t).as_str());
		}
		if search.is_some() {
			query_string.push_str(format!("&search={}", search.unwrap()).as_str());
		}

		client
			.client()
			.get(query_string)
			.send()
			.await
			.expect("Failed to send request")
			.error_for_status()
			.expect("Invalid response")
			.json::<QueryResponse>()
			.await
			.expect("Failed to format response")
	}
}
