use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthorizedClientError {
	#[error("invalid server response")]
	InvalidResponse(#[from] reqwest::Error),
	#[error("invalid header")]
	InvalidHeader (#[from] reqwest::header::InvalidHeaderValue)
}