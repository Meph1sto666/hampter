use reqwest::{self, header::HeaderValue};
use serde_json::json;

#[allow(unused)]
pub struct AuthorizedClient {
	client: reqwest::Client,
	user_agent: String,
	cf_clearance: String,
	auth_token: String,
	refresh_token: String,
	x_app_version: String,
	api_key: String,
}

impl AuthorizedClient {
	pub fn client(&self) -> &reqwest::Client {
		&self.client
	}
	#[allow(dead_code)]
	pub(crate) fn auth_token(&self) -> &str {
		&self.auth_token
	}
}

impl AuthorizedClient {
	pub fn new(
		user_agent: &str,
		cf_clearance: &str,
		auth_token: &str,
		refresh_token: &str,
		x_app_version: &str,
		api_key: &str,
	) -> AuthorizedClient {
		let mut request_headers: reqwest::header::HeaderMap = reqwest::header::HeaderMap::new();
		request_headers.insert(
			"Cookie",
			reqwest::header::HeaderValue::from_str(&format!("cf_clearance={}", cf_clearance))
				.expect("Failed to format cf_clearance cookie"),
		);
		request_headers.insert(
			"User-Agent",
			reqwest::header::HeaderValue::from_str(user_agent).expect("Failed to parse user agent"),
		);
		request_headers.insert(
			"Authorization",
			reqwest::header::HeaderValue::from_str(&format!("Bearer {}", auth_token))
				.expect("Failed to format Bearer"),
		);
		request_headers.insert(
			"x-app-version",
			reqwest::header::HeaderValue::from_str(x_app_version)
				.expect("Failed to parse x-app-version"),
		);

		AuthorizedClient {
			client: reqwest::ClientBuilder::new()
				.cookie_store(true)
				.default_headers(request_headers)
				.build()
				.expect("failed to build client"),
			user_agent: user_agent.to_string(),
			cf_clearance: cf_clearance.to_string(),
			auth_token: auth_token.to_string(),
			refresh_token: refresh_token.to_string(),
			x_app_version: x_app_version.to_string(),
			api_key: api_key.to_string(),
		}
	}

	pub async fn refresh_auth_token(&mut self) {
		#[derive(serde::Deserialize)]
		#[allow(unused)]
		struct Response {
			access_token: String,
			token_type: String,
			expires_in: u32,
			expires_at: u64,
			refresh_token: String,
		}

		let res = self
			.client
			.post("https://auth.janitorai.com/auth/v1/token?grant_type=refresh_token")
			.json(&json!({
			"refresh_token":self.refresh_token
			}))
			.header(
				"apikey",
				HeaderValue::from_str(&self.api_key).expect("Failed to add api key to cookies"),
			)
			.send()
			.await
			.expect("Failed to post token refresh request");

		let parsed_res = res
			.error_for_status()
			.expect("Authorization failed")
			.json::<Response>()
			.await
			.expect("Failed to parse token refresh response");
		self.refresh_token = parsed_res.refresh_token;
		self.auth_token = parsed_res.access_token;
	}
}
