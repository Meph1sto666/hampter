use super::error::HampterError;
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
	/**
	 * Get a list of the clients personas
	 */
	pub async fn list(client: &AuthorizedClient) -> Result<Vec<Persona>, HampterError> {
		Ok(client
			.client()
			.get("https://janitorai.com/hampter/personas/mine")
			.send()
			.await?
			.error_for_status()?
			.json::<Vec<Persona>>()
			.await?)
	}

	/**
	 * Fetch a persona by its ID
	 */
	pub async fn get(
		id: &str,
		client: &AuthorizedClient,
	) -> Result<Persona, HampterError> {
		Ok(client
			.client()
			.get(format!("https://janitorai.com/hampter/personas/{}", id))
			.send()
			.await?
			.error_for_status()?
			.json::<Persona>()
			.await?)
	}
}
