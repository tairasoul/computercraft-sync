use std::{env::current_dir, path::PathBuf, sync::Arc};
#[allow(unused)]
use notify::{EventHandler, Watcher};
use ohkami::{Config, Ohkami, Route, fang::Context};
use parking_lot::RwLock;
use tokio::sync::broadcast::{Receiver, Sender, channel};
use crate::{server::{handlers::{downloads::{handle_download, handle_download_b85, handle_download_base_b85, handle_download_base_libdeflate, handle_download_base_lz4, handle_download_base_sync, handle_download_libdeflate, handle_download_lz4, handle_download_nomin, handle_download_sync}, root::handle_get_root, websocket::handle_subscribe}, util::get_files_for_channel}, structs::Project};
use super::file_watcher::FileChanged;
#[cfg(not(test))]
use super::file_watcher::FileWatcher;

type FileChangedType = (Arc<Sender<FileChanged>>, Arc<Receiver<FileChanged>>);

pub struct SyncServer {
	pub project: Arc<RwLock<Project>>,
	file_changed: FileChangedType
}

impl SyncServer {
	pub fn new(project: Project) -> Self {
		let fc = channel(1000);
		let serv = SyncServer {
			project: Arc::new(RwLock::new(project)),
			file_changed: (Arc::new(fc.0), Arc::new(fc.1))
		};
		serv
	}

	fn get_all_current_files(&self) -> Vec<PathBuf> {
		let mut v = Vec::new();
		let r = self.project.read();
		let project_root = current_dir().unwrap().join(&r.root_dir);
		for channel in &r.items {
			let mut files = get_files_for_channel(&project_root, &channel);
			v.append(&mut files);
		}
		v
	}

	pub fn start_server(&self) -> tokio::task::JoinHandle<()> {
		let _p = self.project.clone();
		let project_root = current_dir().unwrap().join(_p.read().root_dir.clone());
		let s1 = self.file_changed.0.clone();
		let sender = self.file_changed.0.clone();
		let port = _p.read().port.clone();
		let files = self.get_all_current_files();
		#[cfg(not(test))]
		tokio::spawn(async move {
    	use std::collections::HashSet;
			let mut all_existing_files: HashSet<PathBuf> = HashSet::new();
			for file in files {
				all_existing_files.insert(file);
			}
			let handler = FileWatcher::new(sender, all_existing_files);
			let mut watcher = notify::recommended_watcher(handler).unwrap();
			watcher.watch(&project_root, notify::RecursiveMode::Recursive).unwrap();
			std::future::pending::<()>().await;
		});
		tokio::spawn(async move {
			let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await.unwrap();
			let mut cfg = Config::default();
			cfg.keepalive_timeout = 5;
			cfg.websocket_timeout = 14400;
			Ohkami::new((
				Context::new(_p),
				Context::new(s1),
				"/".GET(handle_get_root),
				"/libdeflate.lua".GET(handle_download_libdeflate),
				"/sync.lua".GET(handle_download_sync),
				"/base85.lua".GET(handle_download_b85),
				"/lz4.lua".GET(handle_download_lz4),
				"/base-sync.lua".GET(handle_download_base_sync),
				"/base-libdeflate.lua".GET(handle_download_base_libdeflate),
				"/base-base85.lua".GET(handle_download_base_b85),
				"/base-lz4.lua".GET(handle_download_base_lz4),
				"/download".GET(handle_download),
				"/download-nomin".GET(handle_download_nomin),
				"/subscribe".GET(handle_subscribe)
			)).howl_with(
				cfg,
				listener
			).await;
		})
	}
}

#[cfg(test)]
mod tests {
  use std::{io::Read, time::Duration};
	use flate2::read::DeflateDecoder;
	use crate::{server::{server::SyncServer, util::{chunk_batch, merge}}, structs::{DataSync, Project, ProjectItem, ProjectItemType, RequestType}, util::randstring};

	#[tokio::test]
	async fn get_channels() {
		let project = Project {
			deflate_trickery: None,
			lz_on_deflate: None,
			items: vec![
				ProjectItem {
					channel_name: "hi".to_string(),
					deflate_trickery: None,
					directories: None,
					item_type: ProjectItemType::Library,
					files: None,
					required_channels: None,
					minify: None,
					require_prefix: None,
					prefix_exclusions: None
				},
				ProjectItem {
					channel_name: "hello".to_string(),
					deflate_trickery: None,
					directories: None,
					item_type: ProjectItemType::Resource,
					files: None,
					required_channels: None,
					minify: None,
					require_prefix: None,
					prefix_exclusions: None
				}
			],
			max_uncompressed_request_size: 30000,
			minify: None,
			root_dir: "testdir".to_string(),
			require_prefix: None,
			prefix_exclusions: None,
			port: 8001,
			sync_interval: 1
		};
		let serv = SyncServer::new(project);
		let handle = serv.start_server();

		tokio::time::sleep(Duration::from_millis(1)).await;

		let req = reqwest::get("http://127.0.0.1:8001").await.unwrap();
		let bytes = req.bytes().await.unwrap();
		let mut decompress = DeflateDecoder::new(&bytes[..]);
		let mut output: Vec<u8> = Vec::new();
		let _ = decompress.read_to_end(&mut output).unwrap();
		let outstr: Vec<String> = output.utf8_chunks().map(|e| e.valid().to_string()).collect();
		let full_str = {
			let str = outstr.join("");
			let trimmed = str.trim();
			trimmed.to_string()
		};
		assert_eq!(full_str, "hi - library\nhello - resource".to_string());
		handle.abort();
	}

	#[test]
	fn chunking() {
		let requests = vec![
			RequestType::Resource { data: DataSync { file_path: "hello/hi".to_string(), file_data: randstring(20) } },
			RequestType::Library { data: DataSync { file_path: "hello/hi2".to_string(), file_data: randstring(20) } },
			RequestType::Script { data: DataSync { file_path: "hello/hi3".to_string(), file_data: randstring(20) } },
		];

		let chunked = chunk_batch(requests, 5);

		assert_eq!(chunked.len(), 12);
	}

	#[test]
	fn merging() {
		let requests = vec![
			RequestType::Deletion { files: vec!["hello".to_string(), "hi".to_string()] },
			RequestType::Deletion { files: vec!["hello2".to_string(), "hi2".to_string()] },
			RequestType::Deletion { files: vec!["hello3".to_string(), "hi3".to_string()] },
		];

		let merged = merge(requests);

		let expected = vec![
			RequestType::Deletion { files: vec!["hello".to_string(), "hi".to_string(), "hello2".to_string(), "hi2".to_string(), "hello3".to_string(), "hi3".to_string()] }
		];

		assert!(merged.iter().eq(expected.iter()), "merged is not equal to expected vec");
	}
}