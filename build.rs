// build.rs — invoke flatc to generate FlatBuffers Rust code from
// `protocol/*.fbs` into OUT_DIR.
//
// flatc has a known issue with multi-file Rust output: when file A `include`s
// file B, the generated code emits `use crate::B_generated::*;` but B's types
// actually live at `crate::B_generated::privchat::protocol::*`. We post-process
// each generated file to rewrite the include path correctly.

use std::path::PathBuf;
use std::process::Command;

const SCHEMAS: &[&str] = &[
    "common.fbs",
    "auth.fbs",
    "disconnect.fbs",
    "send.fbs",
    "push.fbs",
    "ping.fbs",
    "subscribe.fbs",
    "publish.fbs",
    "rpc.fbs",
    "transfer.fbs",
    "content.fbs",
];

const SCHEMA_MODULES: &[&str] = &[
    "common_generated",
    "auth_generated",
    "disconnect_generated",
    "send_generated",
    "push_generated",
    "ping_generated",
    "subscribe_generated",
    "publish_generated",
    "rpc_generated",
    "transfer_generated",
    "content_generated",
];

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    for schema in SCHEMAS {
        println!("cargo:rerun-if-changed=protocol/{}", schema);
    }

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");

    let mut cmd = Command::new("flatc");
    cmd.arg("--rust")
        .arg("-o")
        .arg(&out_dir)
        .arg("-I")
        .arg("protocol");
    for schema in SCHEMAS {
        cmd.arg(format!("protocol/{}", schema));
    }

    let status = cmd.status().unwrap_or_else(|e| {
        panic!(
            "Failed to execute flatc: {}\n\
             Install flatc 24.x: `brew install flatbuffers` or \
             https://github.com/google/flatbuffers/releases",
            e
        );
    });

    if !status.success() {
        panic!("flatc compilation failed (exit status: {})", status);
    }

    // Post-process: rewrite `use crate::<mod>_generated::*;` →
    // `use crate::<mod>_generated::privchat::protocol::*;` so cross-file
    // references resolve to the actual nested types.
    for module in SCHEMA_MODULES {
        let path = PathBuf::from(&out_dir).join(format!("{}.rs", module));
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));

        let mut fixed = content;
        for other in SCHEMA_MODULES {
            let needle = format!("use crate::{}::*;", other);
            let replacement = format!("use crate::{}::privchat::protocol::*;", other);
            fixed = fixed.replace(&needle, &replacement);
        }

        std::fs::write(&path, fixed).unwrap_or_else(|e| panic!("write {}: {}", path.display(), e));
    }
}
