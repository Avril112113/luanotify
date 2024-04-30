// NOTE: I am not a rust professional, take every char of this src with a large grain of salt.


use std::{collections::VecDeque, path::Path, sync::{Arc, Mutex}};
use mlua::prelude::*;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};


struct LuaNotify {
	watcher: RecommendedWatcher,
	events: Arc<Mutex<VecDeque<notify::Event>>>,
	filters: Arc<Mutex<Vec<Box<dyn Fn(notify::Event) -> bool + Send>>>>,
}

impl LuaUserData for LuaNotify {
	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_meta_field("__name", "LuaNotify");
	}

	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_function("__tostring", |lua, ud: LuaAnyUserData| {
			// NOTE: The formatting of the pointer differs to Lua.
			// It is *simiar* to LuaJIT but differs in leading zeros after `0x`.
			// Other versions don't have leading `0x` and has all leading zeros depending on pointer bitness.
			return Ok(LuaValue::String(lua.create_string(format!("LuaNotify: {:p}", ud.to_pointer()))?));
		});

		methods.add_method_mut("watch", |lua, this, (path, recursive):(String, bool)| {
			match this.watcher.watch(Path::new(&path), if recursive { RecursiveMode::Recursive } else { RecursiveMode::NonRecursive }) {
				Ok(_) => Ok(LuaMultiValue::from_vec(vec![
					LuaValue::Boolean(true),
					LuaValue::Nil
				])),
				Err(e) => Ok(LuaMultiValue::from_vec(vec![
					LuaValue::Boolean(false),
					LuaValue::String(lua.create_string(e.to_string()).unwrap())
				])),
			}
		});

		methods.add_method_mut("unwatch", |lua, this, path: String| {
			match this.watcher.unwatch(Path::new(&path)) {
				Ok(_) => Ok(LuaMultiValue::from_vec(vec![
					LuaValue::Boolean(true),
					LuaValue::Nil
				])),
				Err(e) => Ok(LuaMultiValue::from_vec(vec![
					LuaValue::Boolean(false),
					LuaValue::String(lua.create_string(e.to_string()).unwrap())
				])),
			}
		});

		methods.add_method_mut("poll", |lua, this, _:()| -> LuaResult<LuaValue> {
			match this.events.lock().unwrap().pop_front() {
				Some(event) => {
					// let norm_paths: Vec<PathBuf> = (&event.paths).iter().map(|path| Path::normalize_virtually(path).unwrap().into_path_buf()).collect();
					// <pain>
					// This is ugly, I hate it, but I don't know rust enough to fix it.
					let type_type = match event.kind {
						notify::EventKind::Access(_) => LuaValue::String(lua.create_string("access")?),
						notify::EventKind::Create(_) => LuaValue::String(lua.create_string("create")?),
						notify::EventKind::Modify(_) => LuaValue::String(lua.create_string("modify")?),
						notify::EventKind::Remove(_) => LuaValue::String(lua.create_string("remove")?),
						_ => LuaValue::String(lua.create_string("unknown")?),
					};
					let type_kind = match event.kind {
						notify::EventKind::Access(kind) => match kind {
							notify::event::AccessKind::Read => LuaValue::String(lua.create_string("read")?),
							notify::event::AccessKind::Open(_) => LuaValue::String(lua.create_string("open")?),
							notify::event::AccessKind::Close(_) => LuaValue::String(lua.create_string("close")?),
							_ => LuaValue::Nil,
						},
						notify::EventKind::Create(kind) => match kind {
							notify::event::CreateKind::File => LuaValue::String(lua.create_string("file")?),
							notify::event::CreateKind::Folder => LuaValue::String(lua.create_string("folder")?),
							_ => LuaValue::Nil,
						},
						notify::EventKind::Modify(kind) => match kind {
							notify::event::ModifyKind::Data(_) => LuaValue::String(lua.create_string("data")?),
							notify::event::ModifyKind::Metadata(_) => LuaValue::String(lua.create_string("metadata")?),
							notify::event::ModifyKind::Name(_) => LuaValue::String(lua.create_string("name")?),
							_ => LuaValue::Nil,
						},
						notify::EventKind::Remove(kind) => match kind {
							notify::event::RemoveKind::File => LuaValue::String(lua.create_string("file")?),
							notify::event::RemoveKind::Folder => LuaValue::String(lua.create_string("folder")?),
							_ => LuaValue::Nil,
						},
						_ => LuaValue::Nil,
					};
					let type_mode = match event.kind {
						notify::EventKind::Access(kind) => match kind {
							notify::event::AccessKind::Open(mode) | notify::event::AccessKind::Close(mode) => match mode {
								notify::event::AccessMode::Execute => LuaValue::String(lua.create_string("execute")?),
								notify::event::AccessMode::Read => LuaValue::String(lua.create_string("read")?),
								notify::event::AccessMode::Write => LuaValue::String(lua.create_string("write")?),
								_ => LuaValue::Nil,
							},
							_ => LuaValue::Nil,
						},
						notify::EventKind::Modify(kind) => match kind {
							notify::event::ModifyKind::Data(change) => match change {
								notify::event::DataChange::Size => LuaValue::String(lua.create_string("size")?),
								notify::event::DataChange::Content => LuaValue::String(lua.create_string("content")?),
								_ => LuaValue::Nil,
							},
							notify::event::ModifyKind::Metadata(meta_kind) => match meta_kind {
								notify::event::MetadataKind::AccessTime => LuaValue::String(lua.create_string("access_time")?),
								notify::event::MetadataKind::WriteTime => LuaValue::String(lua.create_string("write_time")?),
								notify::event::MetadataKind::Permissions => LuaValue::String(lua.create_string("permissions")?),
								notify::event::MetadataKind::Ownership => LuaValue::String(lua.create_string("ownership")?),
								notify::event::MetadataKind::Extended => LuaValue::String(lua.create_string("extended")?),
								_ => LuaValue::Nil,
							},
							notify::event::ModifyKind::Name(mode) => match mode {
								notify::event::RenameMode::To => LuaValue::String(lua.create_string("to")?),
								notify::event::RenameMode::From => LuaValue::String(lua.create_string("from")?),
								notify::event::RenameMode::Both => LuaValue::String(lua.create_string("both")?),
								_ => LuaValue::Nil,
							},
							_ => LuaValue::Nil,
						},
						_ => LuaValue::Nil,
					};
					// </pain>
					Ok(LuaValue::Table(lua.create_table_from(vec![
						("type", type_type),
						("kind", type_kind),
						("mode", type_mode),
						("paths", lua.to_value(&event.paths)?),
						("attrs", lua.to_value(&event.attrs)?),
					]).unwrap()))
				},
				None => Ok(LuaValue::Nil),
			}
		});

		methods.add_method_mut("filter_by_glob", |lua, this, glob: String| {
			let pattern = glob::Pattern::new(&glob);
			if pattern.is_err() {
				return Ok(LuaMultiValue::from_vec(vec![
					LuaValue::Boolean(false),
					LuaValue::String(lua.create_string(pattern.err().unwrap().to_string())?),
				]));
			}
			let pattern = pattern.unwrap();

			this.filters.lock().unwrap().push(Box::new(
				move |event: notify::Event| {
					return event.paths.iter().all(|path: &std::path::PathBuf| pattern.matches_path(path));
				}
			));

			return Ok(LuaMultiValue::from_vec(vec![
				LuaValue::Boolean(true),
				LuaValue::Nil,
			]));
		});
	}
}


fn luanotify_new(_lua: &Lua, _: ()) -> LuaResult<LuaNotify> {
	let events = Arc::new(Mutex::new(VecDeque::new()));
	let filters: Arc<Mutex<Vec<Box<dyn Fn(notify::Event) -> bool + Send>>>> = Arc::new(Mutex::new(Vec::new()));

	let event_cpy = events.clone();
	let filters_cpy = filters.clone();
	let watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
		match res {
			Ok(e) => {
				let locked_filters = filters_cpy.lock().unwrap();
				if locked_filters.len() <= 0 || locked_filters.iter().all(|f| f(e.clone())) {
					event_cpy.lock().unwrap().push_back(e.clone())
				}
			},
			Err(e) => panic!("{}", e),
		}
	}).unwrap();

	return Ok(LuaNotify {
		watcher,
		events,
		filters,
	});
}


// Must be named the same as the output binary (libluanotify.so or luanotify.dll)
#[mlua::lua_module]
fn luanotify(lua: &Lua) -> LuaResult<LuaTable> {
	let exports = lua.create_table()?;
	exports.set("new", lua.create_function(luanotify_new)?)?;
	Ok(exports)
}
