use static_files::resource_dir;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn get_git_hash() -> String {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn main(){
    resource_dir("./dist").build();
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("git_info.rs");
    let mut f = File::create(&dest_path).unwrap();

    let git_hash = get_git_hash();
    writeln!(f, "const GIT_HASH: &str = \"{}\";", git_hash).unwrap();
}