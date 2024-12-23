use crate::auth::AuthorizedClient;
use chrono;
use getters2::Getters;
use serde;

#[derive(serde::Deserialize, serde::Serialize, Getters)]
pub struct Persona {
	id: String,
	name: String,
	avatar: Option<String>,
	appearance: String,
	created_at: chrono::DateTime<chrono::Utc>,
	updated_at: chrono::DateTime<chrono::Utc>,
}

impl Persona {
	pub async fn list(client: &AuthorizedClient) -> Vec<Persona> {
		client
			.client()
			.get("https://janitorai.com/hampter/personas/mine")
			.send()
			.await
			.expect("Failed to send request")
			.error_for_status()
			.expect("Invalid response")
			.json::<Vec<Persona>>()
			.await
			.expect("Failed to parse response")
	}

	pub async fn get(id: &str, client: &AuthorizedClient) -> Persona {
		client
			.client()
			.get(format!("https://janitorai.com/hampter/personas/{}", id))
			.send()
			.await
			.expect("Failed to send request")
			.error_for_status()
			.expect("Invalid response")
			.json::<Persona>()
			.await
			.expect("Failed to parse response")
	}
}
