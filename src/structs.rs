use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone, Debug)]
pub struct File {
	pub path: String,
	#[serde(default)]
	pub cc_path: Option<String>,
	#[serde(default)]
	pub minify: Option<bool>,
	#[serde(default)]
	pub deflate_trickery: Option<bool>,
	#[serde(default)]
	pub bundle: Option<bool>,
	#[serde(default)]
	pub require_prefix: Option<String>,
	#[serde(default)]
	pub prefix_exclusions: Option<Vec<String>>
}

#[derive(Deserialize, Clone, Debug)]
pub struct Directory {
	pub path: String,
	#[serde(default)]
	pub minify: Option<bool>,
	#[serde(default)]
	pub deflate_trickery: Option<bool>,
	#[serde(default)]
	pub require_prefix: Option<String>,
	#[serde(default)]
	pub prefix_exclusions: Option<Vec<String>>
}

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectItemType {
	Resource,
	Library,
	Script
}

impl ToString for ProjectItemType {
	fn to_string(&self) -> String {
		match self {
			Self::Resource => "resource".to_string(),
			Self::Library => "library".to_string(),
			Self::Script => "script".to_string()
		}
	}
}

#[derive(Deserialize, Clone, Debug)]
pub struct ProjectItem {
	#[serde(rename = "type")]
	pub item_type: ProjectItemType,
	#[serde(default)]
	pub files: Option<Vec<File>>,
	pub channel_name: String,
	#[serde(default)]
	pub required_channels: Option<Vec<String>>,
	#[serde(default)]
	pub directories: Option<Vec<Directory>>,
	#[serde(default)]
	pub minify: Option<bool>,
	#[serde(default)]
	pub deflate_trickery: Option<bool>,
	#[serde(default)]
	pub require_prefix: Option<String>,
	#[serde(default)]
	pub prefix_exclusions: Option<Vec<String>>
}

fn get_default_sync_interval() -> u64 {
	1
}

#[derive(Deserialize, Clone, Debug)]
pub struct Project {
	pub root_dir: String,
	pub items: Vec<ProjectItem>,
	pub max_uncompressed_request_size: usize,
	#[serde(default)]
	pub minify: Option<bool>,
	#[serde(default)]
	pub deflate_trickery: Option<bool>,
	#[serde(default)]
	pub require_prefix: Option<String>,
	#[serde(default)]
	pub prefix_exclusions: Option<Vec<String>>,
	#[serde(default)]
	pub lz_on_deflate: Option<bool>,
	pub port: u16,
	#[serde(default = "get_default_sync_interval")]
	pub sync_interval: u64
}

#[derive(PartialEq, Eq)]
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DataSync {
	#[serde(rename = "fp")]
	pub file_path: String,
	#[serde(rename = "fd")]
	pub file_data: String
}

impl Into<Vec<u8>> for RequestType {
	fn into(self) -> Vec<u8> {
		match self {
			Self::Library { data } => Self::vec_from_fsync(data),
			Self::Resource { data } => Self::vec_from_fsync(data),
			Self::Script { data } => Self::vec_from_fsync(data),
			Self::Deletion { files } => Self::vec_from_del(files),
			Self::Chunk { file_data } => Self::vec_from_chunk(file_data)
		}
	}
}

impl RequestType {
	fn vec_from_fsync(data: DataSync) -> Vec<u8> {
		let mut v = Vec::new();
		v.push(0);
		let fp_len = (data.file_path.len() as u32).to_be_bytes();
		let fd_len = (data.file_data.len() as u32).to_be_bytes();
		v.extend_from_slice(&fp_len);
		v.extend_from_slice(&fd_len);
		v.extend_from_slice(data.file_path.as_bytes());
		v.extend_from_slice(data.file_data.as_bytes());
		v
	}

	fn vec_from_del(files: Vec<String>) -> Vec<u8> {
		let mut v = Vec::new();
		v.push(1);
		let v_len = (files.len() as u32).to_be_bytes();
		v.extend_from_slice(&v_len);
		for file in &files {
			let str_len = (file.len() as u32).to_be_bytes();
			v.extend_from_slice(&str_len);
			v.extend_from_slice(file.as_bytes());
		}
		v
	}

	fn vec_from_chunk(chunk: String) -> Vec<u8> {
		let mut v = Vec::new();
		v.push(2);
		let str_len = (chunk.len() as u32).to_be_bytes();
		v.extend_from_slice(&str_len);
		v.extend_from_slice(chunk.as_bytes());
		v
	}
}

#[derive(PartialEq, Eq)]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum RequestType {
	Resource { 
		#[serde(flatten)]
		data: DataSync
	},
	Library { 
		#[serde(flatten)]
		data: DataSync 
	},
	Script { 
		#[serde(flatten)]
		data: DataSync 
	},
	Deletion { 
		#[serde(rename = "f")]
		files: Vec<String> 
	},
	Chunk { 
		#[serde(rename = "fd")]
		file_data: String
	}
}