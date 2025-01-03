use std::io::{self, Write};
use futures::StreamExt;
use hampter::{
	auth::AuthorizedClient,
	types::{chat::{self, MessageChunk}, error::HampterError, profile},
};
use tokio;

#[tokio::main]
async fn main() -> Result<(), HampterError>{
	let client = AuthorizedClient::new(
		"user_agent",
		"cf_clearance",
		"auth_token",
		"refresh_token",
		"x_app_version",
		"api_key",
	)?;

	let chat: chat::Chat = chat::Chat::get(
		0000, // chat id
		&client
	).await?;
	let profile: profile::Profile = profile::Profile::get(&client, None).await?; // none to get your own profile
	let mut lines = chat.generate(
		&client,
		&profile,
		None, // generation mode default to New
		None // message to enhance if using GenerationMode::Suggestion
	).await?;

	while let Some(line) = Some(lines.next()) {
		let line_content = line.await;
		if line_content.is_none() { break; } // check for the stream end
		let json_str = &line_content.unwrap().unwrap();
		let chunk = MessageChunk::from_line(json_str)?;
		if chunk.is_some() {
			print!("{}", chunk.unwrap().content(None));
			let _ = io::stdout().flush();
		}
	}
	Ok(())
}
