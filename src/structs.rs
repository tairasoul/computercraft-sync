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
	#[serde(rename = "filePath")]
	pub file_path: String,
	#[serde(rename = "fileData")]
	pub file_data: String
}

#[derive(PartialEq, Eq)]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum RequestType {
	#[serde(rename = "r")]
	Resource { 
		#[serde(flatten)]
		data: DataSync
	},
	#[serde(rename = "l")]
	Library { 
		#[serde(flatten)]
		data: DataSync 
	},
	#[serde(rename = "s")]
	Script { 
		#[serde(flatten)]
		data: DataSync 
	},
	#[serde(rename = "d")]
	Deletion { 
		files: Vec<String> 
	},
	#[serde(rename = "c")]
	Chunk { 
		file_data: String
	}
}