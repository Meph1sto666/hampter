use super::{misc, profile::Profile};
use crate::auth::AuthorizedClient;
use futures::io::BufReader;
use futures::stream::{self};
use futures::AsyncBufReadExt;
use futures::{
	stream::{MapErr, TryStreamExt},
	Stream,
};
use getters2::Getters;
use serde_json::json;
use std::io;

#[derive(serde::Deserialize, serde::Serialize, Getters)]
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

#[derive(Clone, serde::Deserialize, serde::Serialize, Getters)]
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
	pub fn new(
		mut id: Option<u64>,
		is_bot: bool,
		is_main: bool,
		chat_id: u64,
		content: &str,
		rating: Option<f32>,
	) -> Self {
		Message {
			id: *id.get_or_insert(0),
			created_at: chrono::Utc::now(),
			is_bot: is_bot,
			is_main: is_main,
			chat_id: chat_id,
			message: content.to_string(),
			rating: rating,
		}
	}
}
impl ToString for Message {
	fn to_string(&self) -> String {
		serde_json::to_string(self).expect("Failed to parse message")
	}
}

#[derive(serde::Deserialize, serde::Serialize, Getters)]
pub struct TextDelta {
	role: Option<String>,
	content: String,
}

#[derive(Getters, serde::Deserialize, serde::Serialize)]
pub struct TextChoice {
	index: u64,
	delta: TextDelta,
	logprobs: Option<String>,
	finish_reason: Option<String>,
}

#[derive(Getters, serde::Deserialize, serde::Serialize)]
pub struct MessageChunk {
	id: String,
	object: String,
	created: u64,
	model: String,
	choices: Vec<TextChoice>,
}

impl MessageChunk {
	pub fn from_line(line: &String) -> Option<MessageChunk> {
		if line.is_empty() || line.to_lowercase().contains("data: [done]") {
			return None;
		}
		let parsed = serde_json::from_str(&line.to_string().split_off(6));
		return Some(parsed.expect("Failed to parse response chunk"));
	}

	pub fn content(&self, mut index: Option<usize>) -> &String {
		&self
			.choices
			.get(*index.get_or_insert(0))
			.unwrap()
			.delta
			.content
	}
}

#[derive(serde::Deserialize, serde::Serialize, Getters)]
pub struct ChatInfo {
	id: u32, //615543871,
	is_public: bool,
	summary: String,
	summary_chat_id: Option<String>,
	#[serde(deserialize_with = "misc::u64_from_string")] //, serialize_with=""
	chat_count: u64, // response has String here
	updated_at: chrono::DateTime<chrono::Utc>,
	user_id: String,
	character_id: String,
	persona_id: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Getters)]
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
	/**
	 * Chat actions
	 */
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

	pub async fn delete(id: u64, client: &AuthorizedClient) {
		client
			.client()
			.delete(format!("https://janitorai.com/hampter/chats/{id}", id = id))
			.send()
			.await
			.expect("Failed to send chat delete message")
			.error_for_status()
			.expect("Invalid response");
	}
	
	#[must_use]
	pub async fn create(character_id: &str, client: &AuthorizedClient) -> Chat {
		/**
		 * Open a new chat with a character
		 */
		#[derive(serde::Deserialize, serde::Serialize)]
		struct CreateChatResponse {
			id: u64,
			created_at: chrono::DateTime<chrono::Utc>,
			character_id: String,
			user_id: String,
			is_public: bool,
			summary: String,
			summary_chat_id: Option<String>,
			updated_at: chrono::DateTime<chrono::Utc>,
			chat_count: u64,
			is_deleted: bool,
		}

		let res = client
			.client()
			.post("https://janitorai.com/hampter/chats")
			.json(&json!({
				"character_id": character_id
			}))
			.send()
			.await
			.expect("Failed to create new chat")
			.error_for_status()
			.expect("Invalid response")
			.json::<CreateChatResponse>()
			.await
			.expect("Failed to parse response");
		return Self::get(res.id, client).await;
	}
}

#[derive(PartialEq, Eq)]
pub enum GenerationMode {
	New,
	Suggestion,
	SummaryFull,
	SummaryLast,
	Alternative,
}
impl ToString for GenerationMode {
	fn to_string(&self) -> String {
		match self {
			GenerationMode::New => "NEW".to_string(),
			GenerationMode::Alternative => "ALTERNATIVE".to_string(),
			GenerationMode::Suggestion => "SUGGESTION".to_string(),
			GenerationMode::SummaryFull => "SUMMARY_FULL".to_string(),
			GenerationMode::SummaryLast => "SUMMARY_LAST".to_string(),
		}
	}
}

impl Chat {
	/**
	 * Chat message actions
	 */
	pub async fn generate(
		&self,
		client: &AuthorizedClient,
		profile: &Profile,
		mut mode: Option<GenerationMode>,
		message: Option<Message>,
	) -> futures::io::Lines<
		BufReader<
			stream::IntoAsyncRead<
				MapErr<
					impl Stream<Item = Result<tokio_util::bytes::Bytes, reqwest::Error>>,
					impl FnMut(reqwest::Error) -> io::Error,
				>,
			>,
		>,
	> {
		let mode = mode.get_or_insert(GenerationMode::New);
		if *mode == GenerationMode::Suggestion
			&& message
				.clone()
				.is_some_and(|m: Message| m.message.len() > 20)
		{
			io::Error::new(io::ErrorKind::InvalidInput, "error".to_string()); //Err("Missing message to use auto complete".to_string())
		}
		let response: reqwest::Response = client
			.client()
			.post("https://janitorai.com/generateAlpha")
			.json(&json!({
				"generateMode": mode.to_string(),
				"userConfig": profile.config_ref(),
				"profile": {
					"id": profile.id_ref(),
					"name": profile.name_ref(),
					"user_appearance": "Male", // TODO: get gender somehow?
					"user_name": profile.user_name_ref(),
				},
				"personas": [],
				"chat": {
					"id": self.chat.id,
					"user_id": self.chat.user_id,
					"character_id": self.chat.character_id,
					"summary": self.chat.summary
				},
				"chatMessages": self.chat_messages,
				"forcedPromptGenerationCacheRefetch": { // TODO: get that too somewhere
					"chat": false,
					"character": false,
					"profile": false,
				}
			}))
			.header(reqwest::header::ORIGIN, "https://janitorai.com")
			.send()
			.await
			.expect("Failed to post generation request")
			.error_for_status()
			.expect("Invalid response");

		let reader = response
			.bytes_stream()
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e))
			.into_async_read();
		let decoder = BufReader::new(reader);
		decoder.lines()
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

	pub async fn edit_message(
		&mut self,
		message_id: u64,
		content: &str,
		client: &AuthorizedClient,
	) {
		client
			.client()
			.patch(
				format!(
					"https://janitorai.com/hampter/chats/{chat}/messages/{message}",
					chat = self.chat.id,
					message = message_id
				)
				.to_string(),
			)
			.json(&json!({
				"is_main": true, // so far always has been true in the originals
				"message": content
			}))
			.send()
			.await
			.expect("Failed to send message patch");
		self.chat_messages
			.iter_mut()
			.find(|e: &&mut Message| e.id == message_id)
			.expect("No message with the given ID")
			.message = content.to_string();
	}
	pub fn get_message(&self, message_id: u64) -> std::option::Option<Message> {
		self.chat_messages
			.iter()
			.find(|e: &&Message| e.id == message_id)
			.cloned()
	}

	pub async fn delete_messages(&mut self, message_ids: Vec<u64>, client: &AuthorizedClient) {
		client
			.client()
			.delete(
				format!(
					"https://janitorai.com/hampter/chats/{chat}/messages",
					chat = self.chat.id
				)
				.to_string(),
			)
			.json(&json!({
				"message_ids": message_ids
			}))
			.send()
			.await
			.expect("Failed to send delete message");
		for m_id in message_ids {
			self.chat_messages.remove(
				self.chat_messages
					.iter()
					.position(|message: &Message| message.id == m_id)
					.unwrap(),
			);
		}
	}
}
