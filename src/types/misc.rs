use serde::Deserialize;

pub fn u64_from_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
	D: serde::Deserializer<'de>,
{
	let s: &str = Deserialize::deserialize(deserializer)?;
	s.parse().map_err(serde::de::Error::custom)
}
