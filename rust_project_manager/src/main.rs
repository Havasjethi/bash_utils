#[macro_use]
extern crate serde_derive;

use git2::{RemoteCallbacks, Repository};
use shellexpand;
use toml;

#[derive(Deserialize)]
struct HavasProjectConfig {
    project: Vec<ProjectConfig>,
    defaults: Option<Defaults>,
}

#[derive(Deserialize)]
struct Defaults {
    message: Option<String>,
    no_commit: Option<bool>,
    files: Option<String>,
    should_push: Option<bool>,
    remote: Option<String>,
}

struct InitializedDefault {
    message: String,
    no_commit: bool,
    files: String,
    should_push: bool,
    remote: String,
}

impl InitializedDefault {
    pub fn from_defaults(def: &Option<Defaults>) -> Self {
        if let Some(defaults) = def {
            return InitializedDefault {
                message: defaults
                    .message
                    .clone()
                    .unwrap_or(DEFAULT_MESSAGE.to_string()),
                no_commit: defaults.no_commit.unwrap_or(DEFAULT_NO_COMMIT),
                files: defaults
                    .files
                    .clone()
                    .unwrap_or(DEFAULT_FILES_TO_ADD.to_string()),
                should_push: defaults.should_push.unwrap_or(DEFAULT_SHOULD_PUSH),
                remote: defaults
                    .remote
                    .clone()
                    .unwrap_or(DEFAULT_REMOTE.to_string()),
            };
        } else {
            Self::default()
        }
    }

    pub fn default() -> Self {
        InitializedDefault {
            message: DEFAULT_MESSAGE.to_string(),
            no_commit: DEFAULT_NO_COMMIT,
            files: DEFAULT_FILES_TO_ADD.to_string(),
            should_push: DEFAULT_SHOULD_PUSH,
            remote: DEFAULT_REMOTE.to_string(),
        }
    }
}

#[derive(Deserialize)]
struct ProjectConfig {
    // TODO :: Find usage for this
    name: Option<String>,
    path: Option<String>,
    // TODO :: Create callable function // run:_file_name_ or file://file_path
    message: Option<String>,
    no_commit: Option<bool>,
    // TODO :: ?? Folder matching for spicyness
    // folder_regex: Option<String>,
    files: Option<String>,
    should_push: Option<bool>,
    ignore: Option<bool>,
    remote: Option<String>,
}

const HAVAS_GIT_CONFIG_PATH: &'static str = "/bin/project_file.toml";
const HAVAS_GIT_CONFIG_PATH_KEY: &'static str = "HAVAS_PROJECT_CONFIG";

const DEFAULT_MESSAGE: &'static str = "I too lazy to write commit message";
const DEFAULT_NO_COMMIT: bool = false;
const DEFAULT_SHOULD_PUSH: bool = true;
const DEFAULT_FILES_TO_ADD: &'static str = "*";
const DEFAULT_REMOTE: &'static str = "origin";

fn main() {
    let parsed = get_config();
    let defaults = &parsed.defaults;
    let defaults = InitializedDefault::from_defaults(defaults);

    for project in &parsed.project {
        if project.ignore.unwrap_or(false) {
            println!(
                "Ignoring project: {}",
                project
                    .name
                    .as_ref()
                    .or(project.path.as_ref())
                    .unwrap_or(&"".to_owned())
            );
            continue;
        }

        let path = shellexpand::tilde(&project.path.as_ref().expect("Workdir missing")).to_string();
        let repo = Repository::open(path).expect("Folder not found");

        let commit_message: &str = project.message.as_ref().unwrap_or(&defaults.message);

        let no_commit: bool = project.no_commit.unwrap_or(defaults.no_commit);

        if !no_commit {
            let files = project.files.as_ref().unwrap_or(&defaults.files);
            create_commit(&repo, Some(&files), &commit_message);
        }

        if project.should_push.unwrap_or(defaults.should_push) {
            let target_branch = project.remote.as_ref().unwrap_or(&defaults.remote);
            push(&repo, &target_branch);
        }
    }
}

fn create_commit(repo: &git2::Repository, files_to_add: Option<&str>, commit_message: &str) {
    let head = repo.head().expect("Unable to get index");
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

fn push(repo: &git2::Repository, target: &str) {
    let head = repo.head().expect("Unable to get HEAD");

    let mut remote = repo.find_remote(target).unwrap();

    let branch_ref: &[&str] = &[head.name().unwrap()];

    let mut push_options: git2::PushOptions = git2::PushOptions::new();
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        // Note :: Should it support pwd auth??
        // Gitlab ✓  GitHub ✓
        git2::Cred::ssh_key_from_agent(&username_from_url.unwrap())
    });

    push_options.remote_callbacks(callbacks);
    remote.push(&branch_ref, Some(&mut push_options)).unwrap();
}

fn get_config() -> HavasProjectConfig {
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

    let file = std::fs::read_to_string(&project_file)
        .expect(&format!("Unable to find file: {:?}", &project_file));
    let parsed: HavasProjectConfig = toml::from_str(&file).expect("Wrong file formatting!!");
    parsed
}
