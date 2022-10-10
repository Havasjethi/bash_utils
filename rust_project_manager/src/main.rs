#[macro_use]
extern crate serde_derive;

use git2::{RemoteCallbacks, Repository};
use shellexpand;

use std::fs;
use toml;

#[derive(Deserialize)]
struct Outer {
    project: Vec<ProjectConfig>,
    defaults: Option<Defaults>,
}

#[derive(Deserialize)]
struct Defaults {
    message: Option<String>,
    no_commit: Option<bool>,
    files: Option<String>,
    no_push: Option<bool>,
}

#[derive(Deserialize)]
struct ProjectConfig {
    // TODO ::
    name: Option<String>,
    path: Option<String>,
    message: Option<String>,
    no_commit: Option<bool>,
    // TODO ::
    folder_regex: Option<String>,
    files: Option<String>,
    no_push: Option<bool>,
    ignore: Option<bool>,
}

const HAVAS_GIT_CONFIG_PATH: &'static str = "/bin/project_file.toml";
const HAVAS_GIT_CONFIG_PATH_KEY: &'static str = "HAVAS_PROJECT_CONFIG";

const DEFAULT_MESSAGE: &'static str = "I was lazy";
const DEFAULT_NO_COMMIT: bool = false;
const DEFAULT_FILES_TO_ADD: &'static str = "*";

fn main() {
    let parsed = get_config();
    let defaults = &parsed.defaults;

    for project in &parsed.project {
        if project.ignore.unwrap_or(false) {
            continue;
        }

        let path = shellexpand::tilde(&project.path.as_ref().expect("Workdir missing")).to_string();
        let repo = Repository::open(path).expect("Folder not found");

        let commit_message: &str = project.message.as_ref().map(|e| e.as_str()).unwrap_or(
            defaults
                .as_ref()
                .map(|e| {
                    e.message
                        .as_ref()
                        .map(|e| e.as_str())
                        .unwrap_or(DEFAULT_MESSAGE)
                })
                .unwrap_or(DEFAULT_MESSAGE),
        );

        let no_commit: bool = project.no_commit.unwrap_or(
            defaults
                .as_ref()
                .map(|e| e.no_commit.unwrap_or(DEFAULT_NO_COMMIT))
                .unwrap_or(DEFAULT_NO_COMMIT),
        );

        if !no_commit {
            let files = project.files.as_ref().map(|e| e.as_str()).unwrap_or(
                defaults
                    .as_ref()
                    .map(|e| {
                        e.files
                            .as_ref()
                            .map(|e| e.as_str())
                            .unwrap_or(DEFAULT_FILES_TO_ADD)
                    })
                    .unwrap_or(DEFAULT_FILES_TO_ADD),
            );
            create_commit(&repo, Some(files), commit_message);
        }

        push(&repo, project, &parsed.defaults);
    }
}

fn create_commit(repo: &git2::Repository, files_to_add: Option<&str>, commit_message: &str) {
    let mut head = repo.head().expect("Unable to get index");
    let mut index = repo.index().expect("Unable to get index");

    if let Some(file_matcher) = files_to_add {
        let files_to_add = file_matcher
            // .unwrap_or(&"*".to_string())
            .split(" ")
            .map(|e| e.to_string())
            .collect::<Vec<String>>();

        index.update_all(&files_to_add, None).unwrap();
    }

    let tree_oid = index.write_tree().ok();
    let tree = repo.find_tree(tree_oid.unwrap()).unwrap();
    let head_commit = head.peel_to_commit().unwrap();
    let sign = repo.signature().unwrap();

    repo.commit(
        Some("HEAD"),
        &sign,
        &sign,
        &commit_message,
        &tree,
        &[&head_commit],
    )
    .unwrap();

    index.read(false).unwrap();
    index.write().unwrap();
}

fn push(repo: &git2::Repository, project: &ProjectConfig, defaults: &Option<Defaults>) {
    let mut index = repo.index().expect("Unable to get index");
    let head = repo.head().expect("Unable to get HEAD");

    let defaults = defaults.as_ref().unwrap();

    // Commit message

    let no_push = project.no_push.or(defaults.no_push).unwrap_or(false);
    if no_push {
        return;
    }
    let mut remote = repo.find_remote("origin").unwrap();

    let branch_ref: &[&str] = &[head.name().unwrap()];

    let mut push_options: git2::PushOptions = git2::PushOptions::new();
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        // Gitlab ✓  GitHub ✓
        git2::Cred::ssh_key_from_agent(&username_from_url.unwrap())
    });

    push_options.remote_callbacks(callbacks);
    remote.push(&branch_ref, Some(&mut push_options)).unwrap();
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
