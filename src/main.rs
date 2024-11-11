use std::{env, process};

use funnyexplorerrust::run;

fn main() {
    let cur_path = env::current_dir()
        .inspect_err(|err| {
            eprintln!("{err}");
            process::exit(1)
        })
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    run(cur_path).inspect_err(|err| {
        eprintln!("{err}");
        process::exit(1);
    }).ok();
}
