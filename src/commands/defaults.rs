//! Register commands on the registry.

use std::sync::Arc;
use std::process::Command;
use std::thread;
use std::env;
use std::io::prelude::*;

use commands::{self, CommandFn};
use layout::tree::try_lock_tree;
use lua::{self, LuaQuery};

/// Register the default commands in the API.
///
/// Some of this code will be moved to be called after the config,
/// and will be registered dynamically.
pub fn register_defaults() {
    let mut coms = commands::write_lock();

    let mut register = |name: &'static str, val: CommandFn| {
        coms.insert(name.to_string(), val);
    };

    // Workspace
    register("quit", Arc::new(quit));
    register("launch_terminal", Arc::new(launch_terminal));
    register("launch_dmenu", Arc::new(launch_dmenu));
    register("print_pointer", Arc::new(print_pointer));

    register("dmenu_eval", Arc::new(dmenu_eval));
    register("dmenu_lua_dofile", Arc::new(dmenu_lua_dofile));

    //register("workspace_left", workspace_left);
    //register("workspace_right", workspace_right);

    /// Generate switch_workspace methods and register them in $map
    macro_rules! gen_switch_workspace {
        ( $($b:ident, $n:expr);+ ) => {
            $(fn $b() {
                trace!("Switching to workspace {}", $n);
                if let Ok(mut tree) = try_lock_tree() {
                    tree.switch_to_workspace(&$n.to_string());
                }
            }
            register(stringify!($b), Arc::new($b)); )+
        }
    }


    gen_switch_workspace!(switch_workspace_1, "1";
                          switch_workspace_2, "2";
                          switch_workspace_3, "3";
                          switch_workspace_4, "4";
                          switch_workspace_5, "5";
                          switch_workspace_6, "6";
                          switch_workspace_7, "7";
                          switch_workspace_8, "8";
                          switch_workspace_9, "9";
                          switch_workspace_0, "0");

}

// All of the methods defined should be registered.
#[deny(dead_code)]

fn launch_terminal() {
    let term = env::var("WAYLAND_TERMINAL")
        .unwrap_or("weston-terminal".to_string());

    Command::new("sh").arg("-c")
        .arg(term)
        .spawn().expect("Error launching terminal");
}

fn launch_dmenu() {
    Command::new("sh").arg("-c")
        .arg("dmenu_run")
        .spawn().expect("Error launching terminal");
}

fn print_pointer() {
    use lua;
    use lua::LuaQuery;

    let code = "if wm == nil then print('wm table does not exist')\n\
                elseif wm.pointer == nil then print('wm.pointer table does not exist')\n\
                else\n\
                local x, y = wm.pointer.get_position()\n\
                print('The cursor is at ' .. x .. ', ' .. y)\n\
                end".to_string();
    lua::send(LuaQuery::Execute(code))
        .expect("Error telling Lua to get pointer coords");
}

fn quit() {
    info!("Closing way cooler!!");
    ::rustwlc::terminate();
}

fn dmenu_lua_dofile() {
    thread::Builder::new().name("dmenu_dofile".to_string()).spawn(|| {
        let child = Command::new("dmenu").arg("-p 'Eval Lua file'")
            .spawn().expect("Unable to launch dmenu!");

        {
            // Write \d to stdin to prevent options from being given
            let mut stdin = child.stdin.expect("Unable to access stdin");
            stdin.write_all(b"\n").expect("Unable to write to stdin");
        }

        let mut stdout = child.stdout.expect("Unable to access stdout");
        let mut output = String::new();
        stdout.read_to_string(&mut output).expect("Unable to read stdout");

        lua::send(LuaQuery::ExecFile(output)).expect("unable to contact Lua");
    }).expect("Unable to spawn thread");
}

fn dmenu_eval() {
       thread::Builder::new().name("dmenu_eval".to_string()).spawn(|| {
        let child = Command::new("dmenu").arg("-p 'Eval Lua code'")
            .spawn().expect("Unable to launch dmenu!");
           {
               // Write \d to stdin to prevent options from being given
               let mut stdin = child.stdin.expect("Unable to access stdin");
               stdin.write_all(b"\n").expect("Unable to write to stdin");
           }
           let mut stdout = child.stdout.expect("Unable to access stdout");
           let mut output = String::new();
           stdout.read_to_string(&mut output).expect("Unable to read stdout");

           lua::send(LuaQuery::Execute(output)).expect("Unable to contact Lua");
    }).expect("Unable to spawn thread");
}
