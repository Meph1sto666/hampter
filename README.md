# Hampter

This project is a wrapper for JanitorAI's API to make communication with the server possible without the browser. (Well not completely without because... Cloudflare)

## TODO

> Note that there will neither be an implementation of block or report functionalities for users :p
- [ ] auth
  - [x] create an authorized client
  - [x] refresh authorization token
  - [ ] login (Cloudflare... so maybe not coming)
    - [ ] credentials
    - [ ] google
    - [ ] twitter
    - [ ] discord
  - [ ] sign up (same here)
- [ ] chat
  - [x] create
  - [x] get
  - [ ] list (character-chats)
  - [x] delete
  - [ ] message
    - [x] send
    - [ ] generate (missing a few dynamic parameters)
    - [x] edit
    - [x] delete
    - [ ] rate
- [ ] profile
  - [ ] mine
    - [ ] update
    - [ ] blocked-content
  - [x] get (default to mine)
- [ ] personas
  - [ ] create
  - [x] get
  - [ ] edit
  - [ ] delete
- [ ] character
  - [ ] query (searching)
  - [ ] create
    - [ ] upload image
  - [ ] get
  - [ ] delete
  - [ ] edit
- [ ] tags
  - [ ] get tag list
  - [ ] get tag
- [ ] reviews
- [ ] favourites
- [ ] add crate to cargo

## Examples


### Create an authorized client
```rust
let client: &AuthorizedClient = &AuthorizedClient::new(
	"user_agent",
	// keep in mind that using an agent that is different
	// from the one used to create the cf_clearance token will lead to the requests being blocked
	"cf_clearance_token",
	"bearer_auth_token",
	"refresh_token", // obligatory to request a new bearer token every 30 minutes
	"x_app_version", // not needed for msot requests yet essential for text generation	
	"api_key" // only needed for refreshing the auth token
)
```

### Refresh the auth token

```rust
client.refresh_auth_token(); // client has to be mutable
```

### Generate a response

```rust
use std::io::{self, Write};
use futures::StreamExt;
use hampter::{
	auth::AuthorizedClient,
	types::{chat::{self, MessageChunk}, profile},
};

let chat: chat::Chat = chat::Chat::get(chat_id &client).await;
let profile: profile::Profile = profile::Profile::get(&client, None).await;
let mut lines = chat.generate(&client, &profile, None, None).await;

while let Some(line) = Some(lines.next()) {
	let line_content = line.await;
	if line_content.is_none() { break; } // check for "EOF"
	let json_str = &line_content.unwrap().unwrap();
	let chunk = MessageChunk::from_line(json_str);
	if chunk.is_some() {
		print!("{}", chunk.unwrap().content(None));
		let _ = io::stdout().flush(); // flush for live preview
	}
}
```