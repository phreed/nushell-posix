use nu_plugin::{serve_plugin, MsgPackSerializer};

mod plugin;

fn main() {
    serve_plugin(&mut plugin::PosixPlugin::new(), MsgPackSerializer {})
}
