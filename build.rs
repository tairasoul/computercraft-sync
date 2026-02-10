use std::path::{Path, PathBuf};

use darklua_core::{BundleConfiguration, Configuration, Options, Resources, rules::{ComputeExpression, FilterAfterEarlyReturn, GroupLocalAssignment, PathRequireMode, RemoveComments, RemoveEmptyDo, RemoveIfExpression, RemoveNilDeclaration, RemoveSpaces, RemoveTypes, RemoveUnusedVariable, RemoveUnusedWhile, RenameVariables, Rule, bundle::BundleRequireMode}};

fn main() {
	let libd_minified = {
		let resources = Resources::from_memory();
		resources.write("src/libdeflate.lua", &String::from_utf8(std::fs::read("./lua/libdeflate.lua").unwrap()).unwrap()).unwrap();
		let red: Box<dyn Rule> = Box::new(RemoveEmptyDo::default());
		let faer: Box<dyn Rule> = Box::new(FilterAfterEarlyReturn::default());
		let rnd: Box<dyn Rule> = Box::new(RemoveNilDeclaration::default());
		let rs: Box<dyn Rule> = Box::new(RemoveSpaces::default());
		let ruv: Box<dyn Rule> = Box::new(RemoveUnusedVariable::default());
		let ruw: Box<dyn Rule> = Box::new(RemoveUnusedWhile::default());
		let rif: Box<dyn Rule> = Box::new(RemoveIfExpression::default());
		let rv: Box<dyn Rule> = Box::new(RenameVariables::default().with_function_names());
		let gla: Box<dyn Rule> = Box::new(GroupLocalAssignment::default());
		let ce: Box<dyn Rule> = Box::new(ComputeExpression::default());
		let rc: Box<dyn Rule> = Box::new(RemoveComments::default());
		let rt: Box<dyn Rule> = Box::new(RemoveTypes::default());
		let config = Configuration::empty()
			.with_rule(red)
			.with_rule(faer)
			.with_rule(rnd)
			.with_rule(rs)
			.with_rule(ruv)
			.with_rule(ruw)
			.with_rule(rv)
			.with_rule(gla)
			.with_rule(ce)
			.with_rule(rc)
			.with_rule(rt)
			.with_rule(rif)
			.with_generator(darklua_core::GeneratorParameters::Dense { column_span: usize::MAX });

		darklua_core::process(&resources, 
			Options::new(Path::new("src/libdeflate.lua"))
				.with_configuration(config)
		).expect("could not minify libdeflate.lua").result().expect("could not minify libdeflate.lua (2)");

		resources.get(Path::new("src/libdeflate.lua")).unwrap()
	};

	let b85_minified = {
		let resources = Resources::from_memory();
		resources.write("src/b85.lua", &String::from_utf8(std::fs::read("./lua/base85.lua").unwrap()).unwrap()).unwrap();
		let red: Box<dyn Rule> = Box::new(RemoveEmptyDo::default());
		let faer: Box<dyn Rule> = Box::new(FilterAfterEarlyReturn::default());
		let rnd: Box<dyn Rule> = Box::new(RemoveNilDeclaration::default());
		let rs: Box<dyn Rule> = Box::new(RemoveSpaces::default());
		let ruv: Box<dyn Rule> = Box::new(RemoveUnusedVariable::default());
		let ruw: Box<dyn Rule> = Box::new(RemoveUnusedWhile::default());
		let rv: Box<dyn Rule> = Box::new(RenameVariables::default().with_function_names());
		let gla: Box<dyn Rule> = Box::new(GroupLocalAssignment::default());
		let ce: Box<dyn Rule> = Box::new(ComputeExpression::default());
		let rc: Box<dyn Rule> = Box::new(RemoveComments::default());
		let rt: Box<dyn Rule> = Box::new(RemoveTypes::default());
		let rif: Box<dyn Rule> = Box::new(RemoveIfExpression::default());
		let config = Configuration::empty()
			.with_rule(red)
			.with_rule(faer)
			.with_rule(rnd)
			.with_rule(rs)
			.with_rule(ruv)
			.with_rule(ruw)
			.with_rule(rv)
			.with_rule(gla)
			.with_rule(ce)
			.with_rule(rc)
			.with_rule(rt)
			.with_rule(rif)
			.with_generator(darklua_core::GeneratorParameters::Dense { column_span: usize::MAX });

		darklua_core::process(&resources, 
			Options::new(Path::new("src/b85.lua"))
				.with_configuration(config)
		).expect("could not minify base85.lua").result().expect("could not minify base85.lua (2)");

		resources.get(Path::new("src/b85.lua")).unwrap()
	};

	let lz4_minified = {
		let resources = Resources::from_memory();
		resources.write("src/lz4.lua", &String::from_utf8(std::fs::read("./lua/llz4.lua").unwrap()).unwrap()).unwrap();
		let red: Box<dyn Rule> = Box::new(RemoveEmptyDo::default());
		let faer: Box<dyn Rule> = Box::new(FilterAfterEarlyReturn::default());
		let rnd: Box<dyn Rule> = Box::new(RemoveNilDeclaration::default());
		let rs: Box<dyn Rule> = Box::new(RemoveSpaces::default());
		let ruv: Box<dyn Rule> = Box::new(RemoveUnusedVariable::default());
		let ruw: Box<dyn Rule> = Box::new(RemoveUnusedWhile::default());
		let rv: Box<dyn Rule> = Box::new(RenameVariables::default().with_function_names());
		let gla: Box<dyn Rule> = Box::new(GroupLocalAssignment::default());
		let ce: Box<dyn Rule> = Box::new(ComputeExpression::default());
		let rc: Box<dyn Rule> = Box::new(RemoveComments::default());
		let rt: Box<dyn Rule> = Box::new(RemoveTypes::default());
		let rif: Box<dyn Rule> = Box::new(RemoveIfExpression::default());
		let config = Configuration::empty()
			.with_rule(red)
			.with_rule(faer)
			.with_rule(rnd)
			.with_rule(rs)
			.with_rule(ruv)
			.with_rule(ruw)
			.with_rule(rv)
			.with_rule(gla)
			.with_rule(ce)
			.with_rule(rc)
			.with_rule(rt)
			.with_rule(rif)
			.with_generator(darklua_core::GeneratorParameters::Dense { column_span: usize::MAX });

		darklua_core::process(&resources, 
			Options::new(Path::new("src/lz4.lua"))
				.with_configuration(config)
		).expect("could not minify llz4.lua").result().expect("could not minify llz4.lua (2)");

		resources.get(Path::new("src/lz4.lua")).unwrap()
	};

	let sync_bundled = {
		let resources = Resources::from_memory();
		resources.write("src/msgpack.lua", &String::from_utf8(std::fs::read("./lua/msgpack.lua").unwrap()).unwrap()).unwrap();
		resources.write("src/sync.lua", &String::from_utf8(std::fs::read("./lua/sync.lua").unwrap()).unwrap()).unwrap();
		let red: Box<dyn Rule> = Box::new(RemoveEmptyDo::default());
		let faer: Box<dyn Rule> = Box::new(FilterAfterEarlyReturn::default());
		let rnd: Box<dyn Rule> = Box::new(RemoveNilDeclaration::default());
		let rs: Box<dyn Rule> = Box::new(RemoveSpaces::default());
		let ruv: Box<dyn Rule> = Box::new(RemoveUnusedVariable::default());
		let ruw: Box<dyn Rule> = Box::new(RemoveUnusedWhile::default());
		let rv: Box<dyn Rule> = Box::new(RenameVariables::default().with_function_names());
		let gla: Box<dyn Rule> = Box::new(GroupLocalAssignment::default());
		let ce: Box<dyn Rule> = Box::new(ComputeExpression::default());
		let rc: Box<dyn Rule> = Box::new(RemoveComments::default());
		let rt: Box<dyn Rule> = Box::new(RemoveTypes::default());
		let rif: Box<dyn Rule> = Box::new(RemoveIfExpression::default());
		let config = Configuration::empty()
			.with_bundle_configuration(BundleConfiguration::new(BundleRequireMode::Path(PathRequireMode::new("src"))).with_exclude("/cc-sync/libdeflate"))
			.with_rule(red)
			.with_rule(faer)
			.with_rule(rnd)
			.with_rule(rs)
			.with_rule(ruv)
			.with_rule(ruw)
			.with_rule(rv)
			.with_rule(gla)
			.with_rule(ce)
			.with_rule(rc)
			.with_rule(rt)
			.with_rule(rif)
			.with_generator(darklua_core::GeneratorParameters::Dense { column_span: usize::MAX });

		darklua_core::process(&resources, 
			Options::new(Path::new("src/sync.lua"))
				.with_configuration(config)
		).expect("could not bundle sync.lua").result().expect("could not bundle sync.lua (2)");

		resources.get(Path::new("src/sync.lua")).unwrap()
	};

	let sync_bundled_nomin = {
		let resources = Resources::from_memory();
		resources.write("src/msgpack.lua", &String::from_utf8(std::fs::read("./lua/msgpack.lua").unwrap()).unwrap()).unwrap();
		resources.write("src/sync.lua", &String::from_utf8(std::fs::read("./lua/sync.lua").unwrap()).unwrap()).unwrap();
		let rt: Box<dyn Rule> = Box::new(RemoveTypes::default());
		let config = Configuration::empty()
			.with_bundle_configuration(BundleConfiguration::new(BundleRequireMode::Path(PathRequireMode::new("src"))).with_exclude("/cc-sync/libdeflate"))
			.with_rule(rt);

		darklua_core::process(&resources, 
			Options::new(Path::new("src/sync.lua"))
				.with_configuration(config)
		).expect("could not bundle sync.lua").result().expect("could not bundle sync.lua (2)");

		resources.get(Path::new("src/sync.lua")).unwrap()
	};

	let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
	let sync = out_dir.join("sync.min.lua");
	let sync_nomin = out_dir.join("sync.lua");
	let b85 = out_dir.join("b85.min.lua");
	let libdeflate = out_dir.join("libdeflate.min.lua");
	let lz4 = out_dir.join("lz4.min.lua");
	std::fs::write(&sync, sync_bundled).unwrap();
	std::fs::write(&libdeflate, libd_minified).unwrap();
	std::fs::write(&sync_nomin, sync_bundled_nomin).unwrap();
	std::fs::write(&b85, b85_minified).unwrap();
	std::fs::write(&lz4, lz4_minified).unwrap();
}