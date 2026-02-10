use std::sync::Arc;
use ohkami::{Request, Response, Status, fang::Context};
use parking_lot::RwLock;
use crate::{files::{B85_MINIFIED, BASE_B85, BASE_LIBDEFLATE, BASE_LZ4, BASE_SYNC_BUNDLED, LIBDEFLATE_MINIFIED, LZ4_MINIFIED, SYNC_BUNDLED}, structs::Project};

pub async fn handle_download(req: &Request) -> ohkami::Response {
	if let Some(host) = req.headers.host() {
		let st = format!("local function del(p) if fs.exists(p) then fs.delete(p) end end del(\"/sync.lua\") del(\"/cc-sync/libdeflate.lua\") del(\"/cc-sync/base85.lua\") del(\"/cc-sync/llz4.lua\") shell.run(\"wget http://{0}/sync.lua\")\nshell.run(\"wget http://{0}/libdeflate.lua cc-sync/libdeflate.lua\")\nshell.run(\"wget http://{0}/base85.lua cc-sync/base85.lua\")\nshell.run(\"wget http://{0}/lz4.lua cc-sync/llz4.lua\")", host);
		let bytes: Vec<u8> = st.bytes().collect();
		Response::new(Status::OK).with_payload("text/plain", bytes)
	}
	else {
		Response::BadRequest()
	}
}

pub async fn handle_download_nomin(req: &Request) -> ohkami::Response {
	if let Some(host) = req.headers.host() {
		let st = format!("local function del(p) if fs.exists(p) then fs.delete(p) end end del(\"/sync.lua\") del(\"/cc-sync/libdeflate.lua\") del(\"/cc-sync/base85.lua\") del(\"/cc-sync/llz4.lua\") shell.run(\"wget http://{0}/base-sync.lua sync.lua\")\nshell.run(\"wget http://{0}/base-libdeflate.lua cc-sync/libdeflate.lua\")\nshell.run(\"wget http://{0}/base-base85.lua cc-sync/base85.lua\")\nshell.run(\"wget http://{0}/base-lz4.lua cc-sync/llz4.lua\")", host);
		let bytes: Vec<u8> = st.bytes().collect();
		Response::new(Status::OK).with_payload("text/plain", bytes)
	}
	else {
		Response::BadRequest()
	}
}

pub async fn handle_download_libdeflate(Context(project): Context<'_, Arc<RwLock<Project>>>) -> ohkami::Response {
	let r = project.read();
	if let Some(lz4) = r.lz_on_deflate && lz4 {
		let formatted = format!("return load(require(\"/cc-sync/llz4\").decompress(select(2, require(\"/cc-sync/base85\").decode(\"{}\"))), \"crimes\", \"t\", _G)()", base85::encode(&lz4_flex::compress(LIBDEFLATE_MINIFIED.as_bytes())));
		let bytes: Vec<u8> = formatted.bytes().collect();
		Response::new(Status::OK).with_payload("text/plain", bytes)
	}
	else {
		Response::new(Status::OK).with_payload("text/plain", LIBDEFLATE_MINIFIED.as_bytes())
	}
}

pub async fn handle_download_sync() -> ohkami::Response {
	Response::new(Status::OK).with_payload("text/plain", SYNC_BUNDLED.as_bytes())
}

pub async fn handle_download_base_libdeflate() -> ohkami::Response {
	Response::new(Status::OK).with_payload("text/plain", BASE_LIBDEFLATE.as_bytes())
}

pub async fn handle_download_base_sync() -> ohkami::Response {
	Response::new(Status::OK).with_payload("text/plain", BASE_SYNC_BUNDLED.as_bytes())
}

pub async fn handle_download_b85() -> ohkami::Response {
	Response::new(Status::OK).with_payload("text/plain", B85_MINIFIED.as_bytes())
}

pub async fn handle_download_base_b85() -> ohkami::Response {
	Response::new(Status::OK).with_payload("text/plain", BASE_B85.as_bytes())
}

pub async fn handle_download_lz4() -> ohkami::Response {
	Response::new(Status::OK).with_payload("text/plain", LZ4_MINIFIED.as_bytes())
}

pub async fn handle_download_base_lz4() -> ohkami::Response {
	Response::new(Status::OK).with_payload("text/plain", BASE_LZ4.as_bytes())
}