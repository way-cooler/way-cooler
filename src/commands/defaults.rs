//! Register commands on the registry.

use std::sync::Arc;
use std::process::{Command, Stdio};
use std::thread;
use std::env;
use std::io::prelude::*;

use commands::{self, CommandFn};
use layout::tree::{Direction, try_lock_tree};
use layout::container::{ContainerType, Layout};
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

    register("quit", Arc::new(quit));
    register("launch_terminal", Arc::new(launch_terminal));
    register("launch_dmenu", Arc::new(launch_dmenu));
    register("print_pointer", Arc::new(print_pointer));

    register("dmenu_eval", Arc::new(dmenu_eval));
    register("dmenu_lua_dofile", Arc::new(dmenu_lua_dofile));

    /// Generate switch_workspace methods and register them
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

    //// Generates move_to_workspace methods and registers them
    macro_rules! gen_move_to_workspace {
        ( $($b:ident, $n:expr);+ ) => {
            $(fn $b() {
                trace!("Switching to workspace {}", $n);
                if let Ok(mut tree) = try_lock_tree() {
                    tree.send_active_to_workspace(&$n.to_string());
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

    gen_move_to_workspace!(move_to_workspace_1, "1";
                           move_to_workspace_2, "2";
                           move_to_workspace_3, "3";
                           move_to_workspace_4, "4";
                           move_to_workspace_5, "5";
                           move_to_workspace_6, "6";
                           move_to_workspace_7, "7";
                           move_to_workspace_8, "8";
                           move_to_workspace_9, "9";
                           move_to_workspace_0, "0");

    register("horizontal_vertical_switch", Arc::new(tile_switch));
    register("split_vertical", Arc::new(split_vertical));
    register("split_horizontal", Arc::new(split_horizontal));
    register("focus_left", Arc::new(focus_left));
    register("focus_right", Arc::new(focus_right));
    register("focus_up", Arc::new(focus_up));
    register("focus_down", Arc::new(focus_down));
    register("remove_active", Arc::new(remove_active))

}

// All of the methods defined should be registered.
#[deny(dead_code)]

fn remove_active() {
    if let Ok(mut tree) = try_lock_tree() {
        tree.remove_active();
    }
}

fn tile_switch() {
    if let Ok(mut tree) = try_lock_tree() {
        tree.toggle_active_horizontal();
        tree.layout_active_of(ContainerType::Workspace);
    }
}

fn split_vertical() {
    if let Ok(mut tree) = try_lock_tree() {
        tree.toggle_active_layout(Layout::Vertical);
    }
}

fn split_horizontal() {
    if let Ok(mut tree) = try_lock_tree() {
        tree.toggle_active_layout(Layout::Horizontal);
    }
}

fn focus_left() {
    if let Ok(mut tree) = try_lock_tree() {
        tree.move_focus(Direction::Left);
    }
}

fn focus_right() {
    if let Ok(mut tree) = try_lock_tree() {
        tree.move_focus(Direction::Right);
    }
}

fn focus_up() {
    if let Ok(mut tree) = try_lock_tree() {
        tree.move_focus(Direction::Up);
    }
}

fn focus_down() {
    if let Ok(mut tree) = try_lock_tree() {
        tree.move_focus(Direction::Down);
    }
}

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
        let child = Command::new("dmenu").arg("-p").arg("Eval Lua file")
            .stdin(Stdio::piped()).stdout(Stdio::piped())
            .spawn().expect("Unable to launch dmenu!");

        {
            // Write \d to stdin to prevent options from being given
            let mut stdin = child.stdin.expect("Unable to access stdin");
            stdin.write_all(b"\n").expect("Unable to write to stdin");
        }

        let mut stdout = child.stdout.expect("Unable to access stdout");
        let mut output = String::new();
        stdout.read_to_string(&mut output).expect("Unable to read stdout");

        let result = lua::send(LuaQuery::ExecFile(output))
            .expect("unable to contact Lua").recv().expect("Can't get reply");
        trace!("Lua result: {:?}", result);
    }).expect("Unable to spawn thread");
}

fn dmenu_eval() {
       thread::Builder::new().name("dmenu_eval".to_string()).spawn(|| {
           let child = Command::new("dmenu").arg("-p").arg("Eval Lua code")
               .stdin(Stdio::piped()).stdout(Stdio::piped())
               .spawn().expect("Unable to launch dmenu!");
           {
               // Write \d to stdin to prevent options from being given
               let mut stdin = child.stdin.expect("Unable to access stdin");
               stdin.write_all(b"\n").expect("Unable to write to stdin");
           }
           let mut stdout = child.stdout.expect("Unable to access stdout");
           let mut output = String::new();
           stdout.read_to_string(&mut output).expect("Unable to read stdout");

           let result = lua::send(LuaQuery::Execute(output))
               .expect("Unable to contact Lua").recv().expect("Can't get reply");
           trace!("Lua result: {:?}", result)
    }).expect("Unable to spawn thread");
}
