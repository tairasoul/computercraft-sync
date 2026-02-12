use std::{collections::HashSet, env, io::Write, path::{Path, PathBuf}, sync::Arc};

use darklua_core::{BundleConfiguration, Configuration, Options, Resources, rules::{ComputeExpression, FilterAfterEarlyReturn, GroupLocalAssignment, PathRequireMode, RemoveComments, RemoveEmptyDo, RemoveFunctionCallParens, RemoveIfExpression, RemoveMethodDefinition, RemoveNilDeclaration, RemoveSpaces, RemoveTypes, RemoveUnusedVariable, RemoveUnusedWhile, RenameVariables, Rule, bundle::BundleRequireMode}};
use flate2::{Compression, write::DeflateEncoder};
use lazy_regex::regex_replace_all;
use parking_lot::RwLock;

use crate::{rules::prefix_requires::PrefixRequireRule, structs::{DataSync, Directory, File, Project, ProjectItem, ProjectItemType, RequestType}};

pub struct FileBatcher {
	pub currently_in: Vec<RequestType>
}

impl FileBatcher {
	pub fn new() -> Self {
		Self { currently_in: Vec::new() }
	}

	pub fn add_request(&mut self, req: RequestType) {
		self.currently_in.push(req);
	}

	pub fn retrieve_batch(&mut self) -> Vec<RequestType> {
		let currently_existing = self.currently_in.clone();
		self.currently_in.clear();
		currently_existing
	}
}

pub fn chunk_string(s: &str, chunk_size: usize) -> Vec<&str> {
	let mut chunks = Vec::new();
	let mut char_iter = s.char_indices().peekable();
	
	while char_iter.peek().is_some() {
		let start = char_iter.next().unwrap().0;
			
		for _ in 0..chunk_size - 1 {
			if char_iter.next().is_none() {
				break;
			}
		}
			
		let end = char_iter.peek().map(|(pos, _)| *pos).unwrap_or(s.len());
		chunks.push(&s[start..end]);
	}
	
	chunks
}

pub fn get_associated_item(project: &Arc<RwLock<Project>>, file: &PathBuf, channel: &String) -> Option<(Option<Directory>, Option<File>, ProjectItem)> {
	let r = project.read();
	let items = &r.items;
	let rd = &r.root_dir;
	let root = env::current_dir().unwrap().join(rd);
	for item in items {
		if item.channel_name == channel.clone() {
			if let Some(files) = &item.files {
				for fi in files {
					let f = root.join(fi.path.clone());
					if *file == f {
						return Some((None, Some(fi.clone()), item.clone()));
					}
				}
			}
			else if let Some(dirs) = &item.directories {
				for dir in dirs {
					let d = root.join(&dir.path);
					if file.starts_with(&d) {
						return Some((Some(dir.clone()), None, item.clone()));
					}
				}
			}
		}
	}
	return None;
}

pub fn chunk_batch(batch: Vec<RequestType>, max_uncompressed_request_size: usize) -> Vec<Vec<RequestType>> {
	let mut res: Vec<Vec<RequestType>> = Vec::new();
	let mut current_set: Vec<RequestType> = Vec::new();
	let mut current_size: usize = 0;
	for item in batch {
		match item {
			RequestType::Resource { data } => {
				let len = data.file_data.len();
				if current_size + len > max_uncompressed_request_size {
					if current_set.len() > 0 {
						res.push(current_set);
						current_set = Vec::new();
					}
					current_size = 0;
					if len > max_uncompressed_request_size {
						let chunked = chunk_string(&data.file_data, max_uncompressed_request_size);
						current_set.push(RequestType::Resource { data: DataSync { file_path: data.file_path, file_data: chunked[0].to_string() } });
						res.push(current_set);
						current_set = Vec::new();
						for chunk in chunked.iter().skip(1) {
							current_set.push(RequestType::Chunk { file_data: chunk.to_string() });
							if current_size + chunk.len() >= max_uncompressed_request_size {
								res.push(current_set);
								current_set = Vec::new();
								current_size = 0;
							}
							else {
								current_size += chunk.len();
							}
						}
					}
					else {
						current_size += len;
						current_set.push(RequestType::Resource { data });
					}
				}
				else {
					current_size += len;
					current_set.push(RequestType::Resource { data });
				}
			},
			RequestType::Library { data } => {
				let len = data.file_data.len();
				if current_size + len > max_uncompressed_request_size {
					if current_set.len() > 0 {
						res.push(current_set);
						current_set = Vec::new();
					}
					current_size = 0;
					if len > max_uncompressed_request_size {
						let chunked = chunk_string(&data.file_data, max_uncompressed_request_size);
						current_set.push(RequestType::Library { data: DataSync { file_path: data.file_path, file_data: chunked[0].to_string() } });
						res.push(current_set);
						current_set = Vec::new();
						for chunk in chunked.iter().skip(1) {
							current_set.push(RequestType::Chunk { file_data: chunk.to_string() });
							if current_size + chunk.len() >= max_uncompressed_request_size {
								res.push(current_set);
								current_set = Vec::new();
								current_size = 0;
							}
							else {
								current_size += chunk.len();
							}
						}
					}
					else {
						current_size += len;
						current_set.push(RequestType::Library { data });
					}
				}
				else {
					current_size += len;
					current_set.push(RequestType::Library { data });
				}
			},
			RequestType::Script { data } => {
				let len = data.file_data.len();
				if current_size + len > max_uncompressed_request_size {
					if current_set.len() > 0 {
						res.push(current_set);
						current_set = Vec::new();
					}
					current_size = 0;
					if len > max_uncompressed_request_size {
						let chunked = chunk_string(&data.file_data, max_uncompressed_request_size);
						current_set.push(RequestType::Script { data: DataSync { file_path: data.file_path, file_data: chunked[0].to_string() } });
						res.push(current_set);
						current_set = Vec::new();
						for chunk in chunked.iter().skip(1) {
							current_set.push(RequestType::Chunk { file_data: chunk.to_string() });
							if current_size + chunk.len() >= max_uncompressed_request_size {
								res.push(current_set);
								current_set = Vec::new();
								current_size = 0;
							}
							else {
								current_size += chunk.len();
							}
						}
					}
					else {
						current_size += len;
						current_set.push(RequestType::Script { data });
					}
				}
				else {
					current_size += len;
					current_set.push(RequestType::Script { data });
				}
			},
			RequestType::Deletion { files } => {
				let mut del_vec: Vec<String> = Vec::new();
				for file in files {
					if current_size + file.len() > max_uncompressed_request_size {
						if del_vec.len() > 0 {
							current_set.push(RequestType::Deletion { files: del_vec });
							del_vec = Vec::new();
						}
						if current_set.len() > 0 {
							res.push(current_set);
							current_set = Vec::new();
						}
						current_size = 0;
					}
					current_size += file.len();
					del_vec.push(file);
				}
				current_set.push(RequestType::Deletion { files: del_vec });
			},
			_ => panic!("there should be no chunks inserted already")
		}
	}
	if current_set.len() > 0 {
		res.push(current_set);
	}
	res
}

pub fn merge(batch: Vec<RequestType>) -> Vec<RequestType> {
	let mut res = Vec::new();
	let mut del_vec: Vec<String> = Vec::new();
	for item in batch {
		match item {
			RequestType::Deletion { files } => {
				for file in files {
					del_vec.push(file);
				}
			}
			_ => {
				if del_vec.len() > 0 {
					res.push(RequestType::Deletion { files: del_vec });
					del_vec = Vec::new();
				}
				res.push(item);
			}
		}
	}
	if del_vec.len() > 0 {
		res.push(RequestType::Deletion { files: del_vec });
	}
	res
}

pub fn process_file(file: &PathBuf, root: &PathBuf, item_type: ProjectItemType, minify: bool, deflate: bool, bundle: bool, require_prefix: Option<String>, prefix_exclusions: Option<Vec<String>>) -> String {
	let file_bytes = std::fs::read(file).unwrap();
	let mut content = String::from_utf8(file_bytes).unwrap();
	if item_type != ProjectItemType::Resource {
		if let Some(pfx) = require_prefix.clone() {
			// manually comment out gotos so darklua's parser doesnt screw up
			comment_gotos(&mut content);
			let mut base_exclude = vec!["cc.audio.dfpwm".to_string(), "cc.completion".to_string(), "cc.expect".to_string(), "cc.image.nft".to_string(), "cc.pretty".to_string(), "cc.require".to_string(), "cc.shell.completion".to_string(), "cc.strings".to_string()];
			if let Some(mut exc) = prefix_exclusions.clone() {
				base_exclude.append(&mut exc);
			}
			let rule: Box<dyn Rule> = Box::new(PrefixRequireRule::new(
				pfx, 
				base_exclude
			));
			let cfg = Configuration::empty()
				.with_rule(rule);
			let resources = Resources::from_memory();
			resources.write(file.file_name().unwrap(), &content).unwrap();
			darklua_core::process(&resources, Options::new(Path::new(file.file_name().unwrap())).with_configuration(cfg)).unwrap().result().unwrap();
			content = resources.get(Path::new(file.file_name().unwrap())).unwrap();
			// manually uncomment the commented gotos (hopefully keeping everything functional)
			uncomment_gotos(&mut content);
		}
		if bundle {
			// manually comment out gotos so darklua's parser doesnt screw up
			comment_gotos(&mut content);
			let mut cfg = Configuration::empty();
			let resources = Resources::from_memory();
				cfg = cfg.with_bundle_configuration(
					BundleConfiguration::new(
						BundleRequireMode::Path(
							PathRequireMode::new("")
						)
					)
					.with_exclude("cc.audio.dfpwm")
					.with_exclude("cc.completion")
					.with_exclude("cc.expect")
					.with_exclude("cc.image.nft")
					.with_exclude("cc.pretty")
					.with_exclude("cc.require")
					.with_exclude("cc.shell.completion")
					.with_exclude("cc.strings")
				);
				for file in walkdir::WalkDir::new(root) {
					if let Ok(entry) = file {
						if let Ok(metadata) = entry.metadata() {
							if metadata.is_file() {
								let path = entry.into_path();
								let relative_to_root = path.strip_prefix(root).unwrap();
								let str = relative_to_root.to_string_lossy().to_string().replace("/", ".");
								if let Some(pfx) = require_prefix.clone() {
									let file_content = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
									let mut base_exclude = vec!["cc.audio.dfpwm".to_string(), "cc.completion".to_string(), "cc.expect".to_string(), "cc.image.nft".to_string(), "cc.pretty".to_string(), "cc.require".to_string(), "cc.shell.completion".to_string(), "cc.strings".to_string()];
									if let Some(mut exc) = prefix_exclusions.clone() {
										base_exclude.append(&mut exc);
									}
									let rule: Box<dyn Rule> = Box::new(PrefixRequireRule::new(
										pfx.clone(), 
										base_exclude
									));
									let cfg = Configuration::empty()
										.with_rule(rule);
									let res = Resources::from_memory();
									res.write(path.file_name().unwrap(), &file_content
									.replace("::__continue", "-- ::__continue")
									.replace("goto __continue", "continue -- goto __continue")).unwrap();
									darklua_core::process(&res, Options::new(Path::new(path.file_name().unwrap())).with_configuration(cfg)).unwrap().result().unwrap();
									resources.write(pfx + &str, &res.get(Path::new(path.file_name().unwrap())).unwrap()).unwrap();
								}
								else {
									resources.write(&str, &String::from_utf8(std::fs::read(&path).unwrap()).unwrap()).unwrap();
								}
							}
						}
					}
				}
			resources.write(file.file_name().unwrap(), &content).unwrap();
			darklua_core::process(&resources, Options::new(Path::new(file.file_name().unwrap())).with_configuration(cfg)).unwrap().result().unwrap();
			content = resources.get(Path::new(file.file_name().unwrap())).unwrap();
			// manually uncomment the commented gotos (hopefully keeping everything functional)
			uncomment_gotos(&mut content);
		}
		if minify {
			// manually comment out gotos so darklua's parser doesnt screw up
			comment_gotos(&mut content);
			let cfg = get_darklua_cfg();
			let resources = Resources::from_memory();
			resources.write(file.file_name().unwrap(), &content).unwrap();
			darklua_core::process(&resources, Options::new(Path::new(file.file_name().unwrap())).with_configuration(cfg)).unwrap().result().unwrap();
			content = resources.get(Path::new(file.file_name().unwrap())).unwrap();
			// manually uncomment the commented gotos (hopefully keeping everything functional)
			uncomment_gotos(&mut content);
		}
	}
	if deflate {
		let mut encoder = DeflateEncoder::new(vec![], Compression::best());
		encoder.write(content.as_bytes()).unwrap();
		let res = encoder.finish().unwrap();
		let str = base85::encode(&res);
		let c_str;
		if item_type == ProjectItemType::Resource {
			c_str = format!("return require(\"/cc-sync/libdeflate\").libDeflate:DecompressDeflate(select(2, require(\"/cc-sync/base85\").decode(\"{}\")))", str);
		}
		else {
			c_str = format!("return load(require(\"/cc-sync/libdeflate\").libDeflate:DecompressDeflate(select(2, require(\"/cc-sync/base85\").decode(\"{}\"))))(...)", str);
		}
		if c_str.len() < content.len() {
			content = c_str;
		}
	}
	content
}

pub fn get_full_channel_list(channels: Vec<String>, project: &Arc<RwLock<Project>>, encountered: &mut HashSet<String>) -> Vec<ProjectItem> {
	let r = project.read();
	let mut res = Vec::new();
	let mut cs: Vec<ProjectItem> = Vec::new();
	for channel in channels {
		for item in &r.items {
			if item.channel_name == channel {
				cs.push(item.clone());
				break;
			}
		}
	}
	std::mem::drop(r);
	for channel in cs {
		if encountered.insert(channel.channel_name.clone()) {
			if let Some(req) = &channel.required_channels {
				let mut r = get_full_channel_list(req.clone(), project, encountered);
				res.append(&mut r);
			}
			res.push(channel);
		}
	}
	res
}

pub fn get_files_for_channel(root: &PathBuf, channel: &ProjectItem) -> Vec<PathBuf> {
	let mut v: Vec<PathBuf> = Vec::new();
	let mut discovered: HashSet<PathBuf> = HashSet::new();
	if let Some(dirs) = &channel.directories {
		for dir in dirs {
			let dir_path = root.join(&dir.path);
			let files = walkdir::WalkDir::new(&dir_path);
			for file in files {
				let f = file.unwrap();
				let metadata = f.metadata().unwrap();
				if metadata.is_file() && discovered.insert(f.path().into()) {
					v.push(f.path().into());
				}
			}
		}
	}
	if let Some(files) = &channel.files {
		for file in files {
			let i: PathBuf = root.join(file.path.clone());
			if discovered.insert(i.clone()) {
				v.push(i);
			}
		}
	}
	v
}

pub fn process_tup(tup: Option<(Option<Directory>, Option<File>, ProjectItem)>, batcher: &mut FileBatcher, project_minify: &Option<bool>, project_deflate: &Option<bool>, project_prefix: &Option<String>, project_prefix_exclude: &Option<Vec<String>>, path: &PathBuf, project_root: &PathBuf) {
	if let Some(res) = tup {
		if let Some(dir) = res.0 {
			let item = res.2;
			let minify = {
				if let Some(min_dir) = dir.minify {
					min_dir
				}
				else if let Some(min_item) = item.minify {
					min_item
				}
				else if let Some(min_project) = project_minify {
					*min_project
				} 
				else {
					false
				}
			};
			let deflate_bs = {
				if let Some(min_dir) = dir.deflate_trickery {
					min_dir
				}
				else if let Some(min_item) = item.deflate_trickery {
					min_item
				}
				else if let Some(min_project) = project_deflate {
					*min_project
				} 
				else {
					false
				}
			};
			let file_content = process_file(&path, &project_root, item.item_type, minify, deflate_bs, false, {
					if let Some(prefix) = dir.require_prefix {
						Some(prefix)
					}
					else if let Some(prefix) = item.require_prefix {
						Some(prefix)
					}
					else if let Some(prefix) = project_prefix {
						Some(prefix.clone())
					}
					else {
						None
					}
				},
				{
					if let Some(exclusions) = dir.prefix_exclusions {
						Some(exclusions)
					}
					else if let Some(exclusions) = item.prefix_exclusions {
						Some(exclusions)
					}
					else if let Some(exclusions) = project_prefix_exclude {
						Some(exclusions.clone())
					}
					else {
						None
					}
				}
				);
			let cc_path = path.strip_prefix(&project_root).unwrap();
			match item.item_type {
				ProjectItemType::Resource => {
					let reqtype = RequestType::Resource {
						data: DataSync {
							file_path: cc_path.to_string_lossy().to_string(),
							file_data: file_content
						}
					};
					batcher.add_request(reqtype);
				}
				ProjectItemType::Library => {
					let reqtype = RequestType::Library {
						data: DataSync {
							file_path: cc_path.to_string_lossy().to_string(),
							file_data: file_content
						}
					};
					batcher.add_request(reqtype);
				}
				ProjectItemType::Script => {
					let reqtype = RequestType::Script {
						data: DataSync {
							file_path: cc_path.to_string_lossy().to_string(),
							file_data: file_content
						}
					};
					batcher.add_request(reqtype);
				}
			}
		}
		else if let Some(file) = res.1 {
			let item = res.2;
			let minify = {
				if let Some(min_dir) = file.minify {
					min_dir
				}
				else if let Some(min_item) = item.minify {
					min_item
				}
				else if let Some(min_project) = project_minify {
					*min_project
				}
				else {
					false
				}
			};
			let deflate_bs = {
				if let Some(min_dir) = file.deflate_trickery {
					min_dir
				}
				else if let Some(min_item) = item.deflate_trickery {
					min_item
				}
				else if let Some(min_project) = project_deflate {
					*min_project
				} 
				else {
					false
				}
			};
			let bundle = {
				if let Some(b) = file.bundle {
					b
				}
				else {
					false
				}
			};
			let file_content = process_file(&path, &project_root, item.item_type, minify, deflate_bs, bundle, {
				if let Some(prefix) = file.require_prefix {
					Some(prefix)
				}
				else if let Some(prefix) = item.require_prefix {
					Some(prefix)
				}
				else if let Some(prefix) = project_prefix {
					Some(prefix.clone())
				}
				else {
					None
				}
			},
			{
					if let Some(exclusions) = file.prefix_exclusions {
						Some(exclusions)
					}
					else if let Some(exclusions) = item.prefix_exclusions {
						Some(exclusions)
					}
					else if let Some(exclusions) = project_prefix_exclude {
						Some(exclusions.clone())
					}
					else {
						None
					}
				});
			let cc_path = {
				if let Some(p) = file.cc_path {
					p
				}
				else {
					path.strip_prefix(&project_root).unwrap().to_string_lossy().to_string()
				}
			};
			match item.item_type {
				ProjectItemType::Resource => {
					let reqtype = RequestType::Resource {
						data: DataSync {
							file_path: cc_path,
							file_data: file_content
						}
					};
					batcher.add_request(reqtype);
				}
				ProjectItemType::Library => {
					let reqtype = RequestType::Library {
						data: DataSync {
							file_path: cc_path,
							file_data: file_content
						}
					};
					batcher.add_request(reqtype);
				}
				ProjectItemType::Script => {
					let reqtype = RequestType::Script {
						data: DataSync {
							file_path: cc_path,
							file_data: file_content
						}
					};
					batcher.add_request(reqtype);
				}
			}
		}
	}
}

fn comment_gotos(file_content: &mut String) {
	let goto_replaced = regex_replace_all!("goto (?<label_name>.+)", file_content, |_, label_name| format!("--autocommentedgoto {}", label_name));
	let label_replaced = regex_replace_all!("::(?<label_name>.+)::", &goto_replaced, |_, label_name| format!("--autocommented::{}::", label_name));
	let res = label_replaced.to_string();
	*file_content = res;
}

fn uncomment_gotos(file_content: &mut String) {
	let goto_replaced = regex_replace_all!("--autocommentedgoto (?<label_name>.+)", file_content, |_, label_name| format!(" goto {} ", label_name));
	let label_replaced = regex_replace_all!("--autocommented::(?<label_name>.+)::", &goto_replaced, |_, label_name| format!(" ::{}:: ", label_name));
	let res = label_replaced.to_string();
	*file_content = res;
}

pub fn get_darklua_cfg() -> Configuration {
	let red: Box<dyn Rule> = Box::new(RemoveEmptyDo::default());
	let ce: Box<dyn Rule> = Box::new(ComputeExpression::default());
	let faer: Box<dyn Rule> = Box::new(FilterAfterEarlyReturn::default());
	let rnd: Box<dyn Rule> = Box::new(RemoveNilDeclaration::default());
	let rs: Box<dyn Rule> = Box::new(RemoveSpaces::default());
	let ruv: Box<dyn Rule> = Box::new(RemoveUnusedVariable::default());
	let ruw: Box<dyn Rule> = Box::new(RemoveUnusedWhile::default());
	let rv: Box<dyn Rule> = Box::new(RenameVariables::default().with_function_names());
	let rif: Box<dyn Rule> = Box::new(RemoveIfExpression::default());
	let gla: Box<dyn Rule> = Box::new(GroupLocalAssignment::default());
	let rc: Box<dyn Rule> = Box::new(RemoveComments::default().with_exception("--autocommented"));
	let rt: Box<dyn Rule> = Box::new(RemoveTypes::default());
	let rfcp: Box<dyn Rule> = Box::new(RemoveFunctionCallParens::default());
	let rmd: Box<dyn Rule> = Box::new(RemoveMethodDefinition::default());
	Configuration::empty()
		.with_rule(rmd)
		.with_rule(rfcp)
		.with_rule(rc)
		.with_rule(rt)
		.with_rule(ce)
		.with_rule(red)
		.with_rule(rnd)
		.with_rule(rs)
		.with_rule(ruv)
		.with_rule(ruw)
		.with_rule(rv)
		.with_rule(gla)
		.with_rule(rif)
		.with_rule(faer)
}