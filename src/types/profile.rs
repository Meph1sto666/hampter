use super::{error::AuthorizedClientError, misc};
use crate::auth::AuthorizedClient;
use getters2::Getters;
use serde;
use std::collections::HashMap;

#[derive(serde::Deserialize, serde::Serialize, Getters)]
struct GenerationSettings {
	temperature: f32,    // default to 1.1
	max_new_token: i16,  // default to 500
	context_length: i32, // default to 16384 probably fix
}
impl Default for GenerationSettings {
	fn default() -> Self {
		Self {
			temperature: 1.1,
			max_new_token: 500,
			context_length: 16384,
		}
	}
}

#[derive(serde::Deserialize, serde::Serialize, Getters)]
pub struct Config {
	chat_custom_background_image: String, // default ""
	chat_custom_background_opacity: u8,   // default 10
	chat_custom_background_blur: u8,      // default 0
	chat_custom_foreground_color: String, // default '#ffffff'
	chat_custom_font_size: u8,            // default to 14px
	show_clouds: bool,                    // default false
	show_swords: bool,                    // default false
	generation_settings: GenerationSettings,
	api: String,          // unsure what the default is prob janitor though
	llm_prompt: String,   // ""
	open_ai_mode: String, // also no clue about the default
	#[serde(skip)]
	text_streaming: bool,
	#[serde(skip)]
	immersive_mode: bool,
	#[serde(skip)]
	debug_mode: bool,
	#[serde(skip)]
	use_pygmalion_format: bool,
	#[serde(rename = "openAIKey")]
	open_aikey: Option<String>,
	#[serde(rename = "claudeApiKey")]
	claude_api_key: Option<String>,
	#[serde(rename = "reverseProxyKey")]
	reverse_proxy_key: Option<String>,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			chat_custom_background_image: "".to_string(),
			chat_custom_background_opacity: 10,
			chat_custom_background_blur: 0,
			chat_custom_foreground_color: "#ffffff".to_string(),
			chat_custom_font_size: 14,
			show_clouds: false,
			show_swords: false,
			generation_settings: GenerationSettings::default(),
			api: "".to_string(),
			llm_prompt: "".to_string(),
			open_ai_mode: "api_key".to_string(), // ? was like that in a response *shrug*
			text_streaming: true,
			immersive_mode: false,
			debug_mode: false,
			use_pygmalion_format: true,
			open_aikey: None,
			claude_api_key: None,
			reverse_proxy_key: None,
		}
	}
}

#[derive(serde::Deserialize, serde::Serialize, Getters)]
pub struct Persona {
	id: String,
	name: String,
	avatar: Option<String>,
	appearance: String,
	created_at: chrono::DateTime<chrono::Utc>,
	updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Deserialize, serde::Serialize, Getters)]
pub struct Profile {
	id: String,
	avatar: String,
	name: String,
	user_name: String,
	about_me: String,
	is_verified: bool,
	#[serde(deserialize_with = "misc::u64_from_string")]
	followers_count: u64, // response contains a String
	config: Config,
	profile: String,
	block_list: HashMap<String, Vec<String>>,
	// 	"bots": [],
	// 	"creators": [],
	// 	"tags": [],
	// 	"keywords": []
	// },
	// style: {},
	created_at: chrono::DateTime<chrono::Utc>,
	// user_roles: [],
	personas: Option<Vec<Persona>>,
}
impl Profile {
	/**
	 * Fetch a user profile
	 * If no ID is provided the client profile will be used
	 */
	pub async fn get(
		client: &AuthorizedClient,
		mut id: Option<&str>,
	) -> Result<Profile, AuthorizedClientError> {
		Ok(client
			.client()
			.get(format!(
				"https://janitorai.com/hampter/profiles/{i}",
				i = id.get_or_insert("mine")
			))
			.send()
			.await?
			.error_for_status()?
			.json::<Profile>()
			.await?)
	}
}
