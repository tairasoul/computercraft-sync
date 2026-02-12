use std::{collections::HashSet, env::current_dir, io::Write, sync::Arc, time::Duration};
use flate2::{Compression, write::DeflateEncoder};
use ohkami::{Query, fang::Context, ws::{Message, WebSocket, WebSocketContext}};
use parking_lot::RwLock;
use serde::Deserialize;
use tokio::{sync::{Mutex, broadcast::Sender}, time::interval};
use crate::{server::{file_watcher::FileChanged, util::{FileBatcher, chunk_batch, get_associated_item, get_files_for_channel, get_full_channel_list, merge, process_tup}}, structs::{Project, ProjectItem, RequestType}};

#[derive(Deserialize)]
pub struct SubscribeQuery {
	channels: String
}

pub async fn handle_subscribe(
	Context(project): Context<'_, Arc<RwLock<Project>>>,
	Context(u_mpsc): Context<'_, Arc<Sender<FileChanged>>>,
	ctx: WebSocketContext<'_>,
	Query(channels): Query<SubscribeQuery>
) -> WebSocket {
	let mut rx = u_mpsc.subscribe();
	let arc = project.clone();
	let p = arc.read();
	let project_minify = p.minify.clone();
	let project_deflate = p.deflate_trickery.clone();
	let project_root = p.root_dir.clone();
	let project_prefix = p.require_prefix.clone();
	let project_exclusions = p.prefix_exclusions.clone();
	let sync_interval = p.sync_interval.clone();
	let max_size = p.max_uncompressed_request_size.clone();
	let c: Vec<ProjectItem> = get_full_channel_list(channels.channels.split(",").map(|e| e.to_string()).collect(), &arc, &mut HashSet::new());
	std::mem::drop(p);
	ctx.upgrade(move |conn| async move {
		let conn_arc = Arc::new(Mutex::new(conn));
		let c_list = c;
		let batcher = Arc::new(Mutex::new(FileBatcher::new()));
		let root_path = current_dir().unwrap().join(&project_root);
		for channel in &c_list {
			let files = get_files_for_channel(&root_path, channel);
			for file in files {
				let tup = get_associated_item(&arc, &file, &channel.channel_name);
				let mut batcher_locked = batcher.lock().await;
				process_tup(tup, &mut batcher_locked, &project_minify, &project_deflate, &project_prefix, &project_exclusions, &file, &root_path);
			}
		}
		let batched = batcher.lock().await.retrieve_batch();
		if batched.len() > 0 {
			let merged = merge(batched);
			let chunked = chunk_batch(merged, max_size);
			let mut lock = conn_arc.lock().await;
			for chunk in chunked {
				let byte_vec = {
					let mpd = rmp_serde::encode::to_vec(&chunk).unwrap();
					let mut deflate = DeflateEncoder::new(vec![], Compression::best());
					deflate.write_all(&mpd).unwrap();
					deflate.finish().unwrap()
				};
				lock.send(Message::Binary(byte_vec)).await.unwrap();
			}
		}
		let mut interval = interval(Duration::from_secs(sync_interval));
		loop {
			tokio::select! {
				biased;

				trnsmit = rx.recv() => {
					if let Ok(msg) = trnsmit {
						match msg {
							FileChanged::Changed { path } => {
								for channel in &c_list {
									let tup = get_associated_item(&arc, &path, &channel.channel_name);
									let mut batcher_locked = batcher.lock().await;
									process_tup(tup, &mut batcher_locked, &project_minify, &project_deflate, &project_prefix, &project_exclusions, &path, &root_path);
								}
							}
							FileChanged::Deleted { path } => {
								for channel in &c_list {
									let tup = get_associated_item(&arc, &path, &channel.channel_name);
									if let Some(res) = tup {
										if let Some(_) = res.0 {
											let cc_path = path.strip_prefix(&project_root).unwrap();
											let mut batcher_locked = batcher.lock().await;
											batcher_locked.add_request(RequestType::Deletion { files: vec![cc_path.to_string_lossy().to_string()] });
										}
										else if let Some(file) = res.1 {
											let cc_path = {
												if let Some(p) = file.cc_path {
													p
												}
												else {
													path.strip_prefix(&project_root).unwrap().to_string_lossy().to_string()
												}
											};
											let mut batcher_locked = batcher.lock().await;
											batcher_locked.add_request(RequestType::Deletion { files: vec![cc_path] });
										}
									}
								}
							}
						}
					}
				}
				_ = interval.tick() => {
					let batched = batcher.lock().await.retrieve_batch();
					if batched.len() > 0 {
						let merged = merge(batched);
						let chunked = chunk_batch(merged, max_size);
						let mut lock = conn_arc.lock().await;
						for chunk in chunked {
							let byte_vec = {
								let mpd = rmp_serde::encode::to_vec(&chunk).unwrap();
								let mut deflate = DeflateEncoder::new(vec![], Compression::best());
								deflate.write_all(&mpd).unwrap();
								deflate.finish().unwrap()
							};
							lock.send(Message::Binary(byte_vec)).await.unwrap();
							lock.flush().await.unwrap();
						}
					}
				}
			}
		}
	})
}