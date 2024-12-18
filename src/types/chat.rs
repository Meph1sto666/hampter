use serde::Deserialize;
use serde_json::json;
use crate::auth::AuthorizedClient;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Character {
	id: String,
	name: String,
	chat_name: Option<String>,
	description: String,
	avatar: String,
	is_nsfw: bool,
	is_public: bool,
	is_image_nsfw: bool,
	allow_proxy: bool,
	soundcloud_track_id: Option<String>,
}

#[derive(Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Message {
	id: u64,
	created_at: chrono::DateTime<chrono::Utc>,
	is_bot: bool,
	is_main: bool,
	chat_id: u64,
	message: String,
	rating: Option<f32>, // I assume it's some kind of number not too sure though
}

impl Message {
	pub fn new(mut id: Option<u64>, is_bot: bool, is_main: bool, chat_id: u64, content: &str, rating: Option<f32>) -> Self {
		Message {
			id: *id.get_or_insert(0),
			created_at: chrono::Utc::now(),
			is_bot: is_bot,
			is_main: is_main,
			chat_id: chat_id,
			message: content.to_string(),
			rating: rating
		}
	}
}
impl ToString for Message {
	fn to_string(&self) -> String {
		serde_json::to_string(self).expect("Failed to parse message")
	}
}


#[derive(serde::Deserialize, serde::Serialize)]
pub struct ChatInfo {
	id: u32, //615543871,
	is_public: bool,
	summary: String,
	summary_chat_id: Option<String>,
	#[serde(deserialize_with="u46_from_string")] //, serialize_with=""
	chat_count: u64, // response has String here
	updated_at: chrono::DateTime<chrono::Utc>,
	user_id: String,
	character_id: String,
	persona_id: Option<String>,
}
fn u46_from_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
	D: serde::Deserializer<'de>,
{
	let s: &str = Deserialize::deserialize(deserializer)?;
	s.parse().map_err(serde::de::Error::custom)
}

// fn serialize_u64_as_string<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
// where
//     S: serde_json::Serializer<>,
// {
//     // Convert the u64 to a String and serialize it
//     serializer.serialize_str(&value.to_string())
// }

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Chat {
	chat: ChatInfo,
	character: Character,
	#[serde(rename = "chatMessages")]
	chat_messages: Vec<Message>,
}

impl ToString for Chat {
	fn to_string(&self) -> String {
		serde_json::to_string(self).expect("Failed to serialize Chat")
	}
}

impl Chat {
	pub async fn get(id: u64, client: &AuthorizedClient) -> Chat {
		client
			.client()
			.get(format!("https://janitorai.com/hampter/chats/{}", id))
			.send()
			.await
			.expect("Failed to send get chat request")
			.error_for_status()
			.expect("Invalid response")
			.json::<Chat>()
			.await
			.expect("Failed to parse chat response")
	}

	pub async fn send_message(&mut self, message: Message, client: &AuthorizedClient) -> Message {
		client
			.client()
			.post("https://janitorai.com/hampter/chats/615543871/messages")
			.json(&json!({
				"id": message.id,
				"created_at": message.created_at,
				"is_bot": message.is_bot,
				"is_main": message.is_main,
				"chat_id": message.chat_id,
				"message": message.message,
				"rating": message.rating
			}))
			.send()
			.await
			.expect("Failed to post message")
			.error_for_status()
			.expect("Invalid response")
			.json::<Vec<Message>>()
			.await
			.expect("Failed to parse post response")
			.get(0)
			.expect("Response was empty")
			.clone()
	}
}
