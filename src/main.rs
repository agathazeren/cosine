use ignore::WalkBuilder;
use serde::Deserialize;
use shellexpand;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::fs::File;
use structopt::StructOpt;
use toml;


#[derive(StructOpt)]
#[structopt(name = "cosine", about = "IBM Cloud Object Storage Syncing")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Cmd,
}

#[derive(StructOpt)]
enum Cmd {
    Sync,
}

#[derive(Deserialize)]
struct Config {
    buckets: HashMap<String, Bucket>,
    dirs: Vec<Dir>,
}

#[derive(Deserialize)]
struct Bucket {
    id: String,
}

#[derive(Deserialize)]
struct Dir {
    path: PathBuf,
    bucket: String,
}

fn main() {
    let opt = Opt::from_args();
    let config: Config = toml::from_str(
        &std::fs::read_to_string(&PathBuf::from(shellexpand::tilde("~/.cosine").to_string()))
            .expect("Failed to load config"),
    )
    .expect("Config was not valid toml");

    match opt.cmd {
        Cmd::Sync => sync(&config),
    }
}

fn sync(config: &Config) {
    let mut children = Vec::new();
    for Dir { path, bucket } in &config.dirs {
        let path = shellexpand::tilde(path.to_str().unwrap()).to_string();
        let files = WalkBuilder::new(&path)
            .standard_filters(false)
            .add_custom_ignore_filename(".cosignore")
            .build();
        for file in files {
            let file = PathBuf::from(shellexpand::tilde(file.unwrap().path().to_str().unwrap()).to_string());

            if file.is_dir() {
                continue
            }
            
            children.push(
                Command::new("ibmcloud")
                    .arg("cos")
                    .arg("upload")
                    .arg("--bucket")
                    .arg(&config.buckets[bucket].id)
                    .arg("--key")
                    .arg(&file.strip_prefix(&path).unwrap())
                    .arg("--file")
                    .arg(file)
                    .stdout(File::create("~/cosine.log").unwrap())
                    .stderr(File::create("~/cosine.log").unwrap())
                    .spawn()
                    .unwrap(),
            );
        }
    }

    for mut child in children {
        child.wait().unwrap();
    }
}
