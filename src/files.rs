pub const LIBDEFLATE_MINIFIED: &str = include_str!(concat!(env!("OUT_DIR"), "/libdeflate.min.lua"));
pub const SYNC_BUNDLED: &str = include_str!(concat!(env!("OUT_DIR"), "/sync.min.lua"));
pub const B85_MINIFIED: &str = include_str!(concat!(env!("OUT_DIR"), "/b85.min.lua"));
pub const LZ4_MINIFIED: &str = include_str!(concat!(env!("OUT_DIR"), "/lz4.min.lua"));
pub const BASE_LIBDEFLATE: &str = include_str!("../lua/libdeflate.lua");
pub const BASE_SYNC_BUNDLED: &str = include_str!(concat!(env!("OUT_DIR"), "/sync.lua"));
pub const BASE_B85: &str = include_str!("../lua/base85.lua");
pub const BASE_LZ4: &str = include_str!("../lua/llz4.lua");