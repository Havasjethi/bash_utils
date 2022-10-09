#[macro_use]
extern crate serde_derive;
// use serde_derive::Deserialize;

use git2::{RemoteCallbacks, Repository};
use shellexpand;

use std::{fs, os::unix::process::CommandExt};
use toml;

#[derive(Deserialize, Debug)]
struct Outer {
    project: Vec<ProjectConfig>,
}

#[derive(Deserialize, Debug)]
struct ProjectConfig {
    name: Option<String>,
    path: Option<String>,
    message: Option<String>,
    no_commit: Option<bool>,
    folder_regex: Option<String>,
    files: Option<String>,
}

const HAVAS_GIT_PROGRAM: &'static str = "havas_git.sh";
const HAVAS_GIT_CONFIG_PATH: &'static str = "/bin/project_file.toml";
const HAVAS_GIT_CONFIG_PATH_KEY: &'static str = "HAVAS_PROJECT_CONFIG";

fn main() {
    let parsed = get_config();

    for project in parsed.project {
        // do_operation(project);
        do_shit_git(project);
    }
}

fn do_shit_git(mut project: ProjectConfig) {
    let path = shellexpand::tilde(&project.path.as_ref().expect("Workdir missing")).to_string();
    let repo = Repository::open(path).expect("Folder not found");
    let mut index = repo.index().expect("cannot get index");
    let commit_message = "AM I Crazy?";
    let head = repo.head().unwrap();

    // If commits enabled -> Commit selected changes
    if project.no_commit.as_ref().is_none() {
        let stuff = project
            .files
            .as_ref()
            .unwrap()
            .split(" ")
            .map(|e| e.to_string())
            .collect::<Vec<String>>();

        // index
        // .add_all(&stuff, git2::IndexAddOption::DEFAULT, None)
        // .unwrap();
        index.update_all(&stuff, None).unwrap();
        let tree_oid = index.write_tree().ok();

        let tree = repo.find_tree(tree_oid.unwrap()).unwrap();
        let head_commit = head.peel_to_commit().unwrap();
        let sign = repo.signature().unwrap();

        let commit = repo
            .commit(
                Some("HEAD"),
                &sign,
                &sign,
                commit_message,
                &tree,
                &[&head_commit],
            )
            .unwrap();
        // index.clear().unwrap();
        // index.write().unwrap();

        index.read(false);
        index.write();
    }

    let mut remote = repo.find_remote("origin").unwrap();

    let x: &[&str] = &[head.name().unwrap()];

    let mut push_options: git2::PushOptions = git2::PushOptions::new();

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        dbg!((&_url, &username_from_url, &_allowed_types));
        // git2::Cred::ssh_key(username, publickey, privatekey, passphrase)

        // git2::Cred::ssh_key(
        // "havas", // username_from_url.unwrap(),
        // None,
        // std::path::Path::new(&format!("{}/.ssh/id_rsa", std::env::var("HOME").unwrap())),
        // None,
        // )

        // Gitlab ✓  GitHub X
        let res = git2::Cred::ssh_key_from_agent(&username_from_url.unwrap());
        let private = std::path::Path::new("/home/havas/.ssh/havas_github");
        let public = std::path::Path::new("/home/havas/.ssh/havas_github.pub");
        // let res = git2::Cred::ssh_key(&username_from_url.unwrap(), Some(public), private, None);
        // let res = git2::Cred::default();
        // let res = git2::Cred::username(&username_from_url.unwrap());
        // let res = git2::Cred::ssh_key_from_memory(
        // &username_from_url.unwrap(),
        // Some("/home/havas/.ssh/havas_github"),
        // "/home/havas/.ssh/havas_github.pub",
        // None,
        // );
        dbg!(res.is_ok());

        res
    });

    push_options.remote_callbacks(callbacks);
    remote.push(&x, Some(&mut push_options)).unwrap();
}

fn do_operation(project: ProjectConfig) {
    let workdir = shellexpand::tilde(&project.path.unwrap()).to_string();
    let mut args = vec![
        "--absolute_index".to_string(),
        "--workdir".to_string(),
        workdir.clone(),
    ];

    if let Some(msg) = project.message {
        args.push("--message".to_string());
        args.push(format!("\"{}\"", msg));
    }
    if let Some(msg) = project.no_commit {
        if msg {
            args.push("--no_commit".to_string());
        }
    } else if let Some(files) = project.files {
        args.push("--files".to_string());
        args.push(format!("{}/{}", workdir, files));
    }

    let mut process = std::process::Command::new(HAVAS_GIT_PROGRAM);
    dbg!(&args);
    process.args(args);
    process.exec();
}

fn get_config() -> Outer {
    let project_file = std::env::args_os()
        .skip(1)
        // .map(|e| e.into())
        .collect::<Vec<std::ffi::OsString>>()
        .pop()
        .or_else(|| std::env::var_os(HAVAS_GIT_CONFIG_PATH_KEY))
        .unwrap_or_else(|| {
            let mut s = std::env::var_os("HOME").expect("HOME env variable is missing");
            s.push(HAVAS_GIT_CONFIG_PATH);
            s
        });

    let file = fs::read_to_string(&project_file)
        .expect(&format!("Unable to find file: {:?}", &project_file));
    let parsed: Outer = toml::from_str(&file).expect("Wrong file formatting!!");
    parsed
}
