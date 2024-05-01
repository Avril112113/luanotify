// NOTE: I am not a rust professional, take every char of this src with a large grain of salt.


use std::{collections::VecDeque, path::Path, sync::{Arc, Mutex}};
use mlua::prelude::*;
use notify::{event::{AccessKind, AccessMode, CreateKind, DataChange, MetadataKind, ModifyKind, RemoveKind, RenameMode}, EventKind, RecommendedWatcher, RecursiveMode, Watcher};


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
					// Paths are not being normalized due to build issues on linux with the prior library.
					// let norm_paths: Vec<PathBuf> = (&event.paths).iter().map(|path| Path::normalize_virtually(path).unwrap().into_path_buf()).collect();

					let type_type = match event.kind {
						EventKind::Access(_) => Some("access"),
						EventKind::Create(_) => Some("create"),
						EventKind::Modify(_) => Some("modify"),
						EventKind::Remove(_) => Some("remove"),
						_ => None,
					};
					let type_type = if let Some(s) = type_type { LuaValue::String(lua.create_string(s)?) } else { LuaValue::Nil };

					let type_kind = match event.kind {
						EventKind::Access(AccessKind::Read) => Some("read"),
						EventKind::Access(AccessKind::Open(_)) => Some("open"),
						EventKind::Access(AccessKind::Close(_)) => Some("close"),
						
						EventKind::Create(CreateKind::File) => Some("file"),
						EventKind::Create(CreateKind::Folder) => Some("folder"),
						
						EventKind::Modify(ModifyKind::Data(_)) => Some("data"),
						EventKind::Modify(ModifyKind::Metadata(_)) => Some("metadata"),
						EventKind::Modify(ModifyKind::Name(_)) => Some("name"),
						
						EventKind::Remove(RemoveKind::File) => Some("file"),
						EventKind::Remove(RemoveKind::Folder) => Some("folder"),

						_ => None,
					};
					let type_kind = if let Some(s) = type_kind { LuaValue::String(lua.create_string(s)?) } else { LuaValue::Nil };

					let type_mode = match event.kind {
						EventKind::Access(kind) => match kind {
							AccessKind::Open(AccessMode::Execute) => Some("execute"),
							AccessKind::Open(AccessMode::Read) => Some("read"),
							AccessKind::Open(AccessMode::Write) => Some("write"),

							AccessKind::Close(AccessMode::Execute) => Some("execute"),
							AccessKind::Close(AccessMode::Read) => Some("read"),
							AccessKind::Close(AccessMode::Write) => Some("write"),

							_ => None,
						},
						EventKind::Create(kind) => match kind {
							CreateKind::File => Some("file"),
							CreateKind::Folder => Some("folder"),
							
							_ => None,
						},
						EventKind::Modify(kind) => match kind {
							ModifyKind::Data(DataChange::Size) => Some("size"),
							ModifyKind::Data(DataChange::Content) => Some("content"),

							ModifyKind::Metadata(MetadataKind::AccessTime) => Some("access_time"),
							ModifyKind::Metadata(MetadataKind::WriteTime) => Some("write_time"),
							ModifyKind::Metadata(MetadataKind::Permissions) => Some("permissions"),
							ModifyKind::Metadata(MetadataKind::Ownership) => Some("ownership"),
							ModifyKind::Metadata(MetadataKind::Extended) => Some("extended"),

							ModifyKind::Name(RenameMode::To) => Some("to"),
							ModifyKind::Name(RenameMode::From) => Some("from"),
							ModifyKind::Name(RenameMode::Both) => Some("both"),

							_ => None,
						},
						EventKind::Remove(kind) => match kind {
							RemoveKind::File => Some("file"),
							RemoveKind::Folder => Some("folder"),
							
							_ => None,
						},
						_ => None,
					};
					let type_mode = if let Some(s) = type_mode { LuaValue::String(lua.create_string(s)?) } else { LuaValue::Nil };

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
