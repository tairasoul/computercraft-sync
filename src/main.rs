use std::env::current_dir;

use ron::Options;
use tokio::runtime::Builder;

use crate::{server::server::SyncServer, structs::Project};

mod files;
mod server;
mod structs;
mod util;
mod rules;

async fn main_fn() {
	let cd = current_dir().unwrap();
	let cfg_path = cd.join("project.ron");
	if !std::fs::exists(&cfg_path).unwrap() {
		println!("project.ron not found in current directory");
		return;
	}

	let options = Options::default()
		.with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);

	let contents = std::fs::read(&cfg_path).unwrap();

	let project = options.from_bytes::<Project>(&contents).expect("failed to deserialize project.ron");

	let root_dir = cd.join(&project.root_dir);

	if !std::fs::exists(&root_dir).unwrap() {
		println!("could not find {} relative to current directory", root_dir.to_string_lossy());
		return;
	}

	for item in &project.items {
		if item.channel_name.contains(char::is_whitespace) {
			println!("channel name \"{}\" contains whitespace, remove any whitespace present", item.channel_name);
			return;
		}
	}

	let server = SyncServer::new(project);

	server.start_server().await.unwrap();
}

fn main() {
	let rt = Builder::new_multi_thread()
    .thread_stack_size(16 * 1024 * 1024) // 16 mb stack because darklua might use quite a bit apparently
		.enable_all()
    .build()
    .unwrap();

	rt.block_on(main_fn());
}