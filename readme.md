# LuaNotify
A wrapper around rust's [notify](https://docs.rs/notify) crate.  
This provides a cross-platform way to receive filesystem notifications/events.  
All Lua versions should be supported.

See the [example](example.lua) for how to use it.  

To build run `cargo build --features=mlua/luajit`.  
*Requires rust/cargo*  
