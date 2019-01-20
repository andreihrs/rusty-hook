use super::*;

#[test]
fn get_root_directory_path_returns_path() {
    let exp = "/usr/me/foo";
    let run_command = |cmd: &str| {
        if cmd == "git rev-parse --show-toplevel" {
            Ok(String::from(exp))
        } else {
            Ok(String::from(""))
        }
    };
    let act = get_root_directory_path(run_command);
    assert_eq!(act.unwrap(), exp);
}

#[test]
fn get_root_directory_path_returns_err() {
    let exp_err = "Ah!";
    let run_command = |_cmd: &str| Err(String::from(exp_err));
    let act = get_root_directory_path(run_command);
    assert_eq!(act, Err(String::from(exp_err)));
}

#[test]
fn get_hooks_directory_returns_path() {
    let exp = "/.git/hooks";
    let run_command = |cmd: &str| {
        if cmd == "git rev-parse --git-path hooks" {
            Ok(String::from(exp))
        } else {
            Ok(String::from(""))
        }
    };
    let act = get_hooks_directory(run_command);
    assert_eq!(act.unwrap(), exp);
}

#[test]
fn get_hooks_directory_returns_err() {
    let exp_err = "failed";
    let run_command = |_cmd: &str| Err(String::from(exp_err));
    let act = get_hooks_directory(run_command);
    assert_eq!(act, Err(String::from(exp_err)));
}

#[test]
fn create_hook_files_fails_when_hooks_directory_unknown() {
    let exp_err = "Failure determining git hooks directory";
    let run_command = |_cmd: &str| Err(String::from(""));
    let write_file = |_path: &str, _contents: &str| Ok(());
    let result = create_hook_files(run_command, write_file, "");
    assert_eq!(result, Err(String::from(exp_err)));
}

#[test]
fn create_hook_files_fails_when_hook_write_fails() {
    let exp_err = "Fatal error encountered while trying to create git hook files";
    let run_command = |_cmd: &str| Ok(String::from("/usr/repos/foo/.git/hooks"));
    let write_file = |_path: &str, _contents: &str| Err(String::from(""));
    let result = create_hook_files(run_command, write_file, "");
    assert_eq!(result, Err(String::from(exp_err)));
}

const EXP_HOOK_NAMES: [&str; 19] = [
    "applypatch-msg",
    "pre-applypatch",
    "post-applypatch",
    "pre-commit",
    "prepare-commit-msg",
    "commit-msg",
    "post-commit",
    "pre-rebase",
    "post-checkout",
    "post-merge",
    "pre-push",
    "pre-receive",
    "update",
    "post-receive",
    "post-update",
    "push-to-checkout",
    "pre-auto-gc",
    "post-rewrite",
    "sendemail-validate",
];

#[test]
fn should_have_create_hooks() {
    for (&exp_hook, &act_hook) in EXP_HOOK_NAMES.iter().zip(HOOK_NAMES.iter()) {
        assert_eq!(exp_hook, act_hook);
    }
}

#[test]
fn create_hook_files_creates_all_hooks() {
    let version = env!("CARGO_PKG_VERSION");
    let root_dir = "/usr/repos/foo";
    let git_hooks = ".git/hooks";
    let exp_contents = &String::from(HOOK_FILE_TEMPLATE).replace("{{VERSION}}", version);
    let run_command = |_cmd: &str| Ok(String::from(git_hooks));
    let write_file = |path: &str, contents: &str| {
        let act_hook = &&path[(path.rfind('/').unwrap() + 1)..];
        let exp_hook = EXP_HOOK_NAMES
            .iter()
            .find(|&n| n == act_hook)
            .unwrap();
        let exp_path = &format!("{}/{}/{}", root_dir, git_hooks, exp_hook);
        assert_eq!(exp_path, path);
        assert_eq!(exp_contents, contents);
        Ok(())
    };
    let result = create_hook_files(run_command, write_file, root_dir);
    assert_eq!(result, Ok(()));
}