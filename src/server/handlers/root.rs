use std::{io::Write, sync::Arc};

use flate2::{Compression, write};
use ohkami::{Response, Status, fang::Context};
use parking_lot::RwLock;

use crate::structs::{Project, ProjectItem};

pub async fn handle_get_root(
	Context(project): Context<'_, Arc<RwLock<Project>>>
) -> ohkami::Response {
	let r = project.read();
	let channels: Vec<&ProjectItem> = r.items.iter().collect();
	let mut compressed = write::DeflateEncoder::new(Vec::new(), Compression::best());
	for channel in channels {
		compressed.write_all(channel.channel_name.as_bytes()).unwrap();
		compressed.write_all(" - ".as_bytes()).unwrap();
		compressed.write_all(channel.item_type.to_string().as_bytes()).unwrap();
		compressed.write_all("\n".as_bytes()).unwrap();
	}
	Response::new(Status::OK).with_payload("application/octet-stream", compressed.finish().unwrap())
}