use thiserror::Error;

#[derive(Error, Debug)]
pub enum HampterError {
	#[error("invalid server response")]
	InvalidResponse(#[from] reqwest::Error),
	#[error("invalid header")]
	InvalidHeader (#[from] reqwest::header::InvalidHeaderValue),
	#[error("serialization failed")]
	FailedSerialization (#[from] serde_json::Error),
}