use once_cell::sync::Lazy;
use std::{env, fs, iter, path::PathBuf, sync::Mutex};

static TO_REMOVE: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(<_>::default);

/// Remove [`process_dir`] if exists and set to auto-delete when created.
pub fn clean() {
    if let Some(dir) = TO_REMOVE.lock().unwrap().take() {
        fs::remove_dir_all(dir).expect("failed to remove temp-dir");
    }
}

/// Return a temporary directory that is distinct per process/run.
///
/// Configured --temp-dir is used as a parent or, if not set, the current working dir.
///
/// `delete_on_exit`: If true on first call auto delete this dir on process exit.
pub fn process_dir(conf_parent: Option<PathBuf>, delete_on_exit: bool) -> PathBuf {
    static SUBDIR: Lazy<String> = Lazy::new(|| {
        let mut subdir = String::from(".vimg-");
        subdir.extend(iter::repeat_with(fastrand::alphanumeric).take(12));
        subdir
    });

    let mut temp_dir =
        conf_parent.unwrap_or_else(|| env::current_dir().expect("current working directory"));
    temp_dir.push(&*SUBDIR);

    if !temp_dir.exists() && delete_on_exit {
        TO_REMOVE.lock().unwrap().replace(temp_dir.clone());
        fs::create_dir_all(&temp_dir).expect("failed to create temp-dir");
    }

    temp_dir
}
