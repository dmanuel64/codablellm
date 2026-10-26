use clap::CommandFactory;
use std::env;
use std::io::Error;
use std::path::PathBuf;

#[path = "src/cli.rs"]
mod cli;
use cli::Cli;

fn main() -> Result<(), Error> {
    println!("cargo:rerun-if-changed=src/cli.rs");
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = match env::var_os("OUT_DIR") {
        Some(out) => PathBuf::from(out),
        None => return Ok(()),
    };

    let cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    std::fs::write(out_dir.join("codablellm.1"), buffer)?;
    Ok(())
}
