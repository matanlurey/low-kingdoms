use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use super::{StorageError, Store};

pub(super) fn default_store() -> Result<Box<dyn Store>, StorageError> {
    // Native smoke fixtures own isolated slots and must never replace a developer's save.
    #[cfg(feature = "test-support")]
    if let Some(directory) = env_path("LOW_KINGDOMS_TEST_DATA_DIR") {
        return Ok(Box::new(FileStore {
            path: test_directory(directory)?.join("active-v1.json"),
        }));
    }
    let directory = app_data_directory(
        env::consts::OS,
        env_path("HOME"),
        env_path("LOCALAPPDATA"),
        env_path("XDG_DATA_HOME"),
    )?;
    Ok(Box::new(FileStore {
        path: directory.join("active-v1.json"),
    }))
}

#[cfg(feature = "test-support")]
fn test_directory(directory: PathBuf) -> Result<PathBuf, StorageError> {
    if directory.is_absolute() {
        Ok(directory)
    } else {
        Err(StorageError::Unavailable(
            "LOW_KINGDOMS_TEST_DATA_DIR must name an absolute fixture directory.".into(),
        ))
    }
}

fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn app_data_directory(
    os: &str,
    home: Option<PathBuf>,
    local_app_data: Option<PathBuf>,
    xdg_data_home: Option<PathBuf>,
) -> Result<PathBuf, StorageError> {
    let directory = match os {
        "windows" => local_app_data
            .filter(|path| path.is_absolute())
            .ok_or_else(|| {
                StorageError::Unavailable(
                    "LOCALAPPDATA must name an absolute user directory.".into(),
                )
            })?,
        "macos" => absolute_home(home)?.join("Library/Application Support"),
        _ => match xdg_data_home.filter(|path| path.is_absolute()) {
            Some(directory) => directory,
            // The XDG specification ignores a relative XDG_DATA_HOME value.
            None => absolute_home(home)?.join(".local/share"),
        },
    };
    Ok(directory.join("low-kingdoms"))
}

fn absolute_home(home: Option<PathBuf>) -> Result<PathBuf, StorageError> {
    home.filter(|path| path.is_absolute()).ok_or_else(|| {
        StorageError::Unavailable("HOME must name an absolute user directory.".into())
    })
}

pub(super) fn fresh_seed() -> Result<u64, StorageError> {
    let mut bytes = [0_u8; 8];
    getrandom::fill(&mut bytes).map_err(|_| StorageError::Entropy)?;
    Ok(u64::from_le_bytes(bytes))
}

struct FileStore {
    path: PathBuf,
}

impl Store for FileStore {
    fn read(&mut self) -> Result<Option<String>, StorageError> {
        match fs::read(&self.path) {
            Ok(data) => String::from_utf8(data)
                .map(Some)
                .map_err(|_| StorageError::Encoding),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(StorageError::Read(error.to_string())),
        }
    }

    fn write(&mut self, data: &str) -> Result<(), StorageError> {
        let directory = self.path.parent().ok_or_else(|| {
            StorageError::Unavailable("The save location has no parent directory.".into())
        })?;
        fs::create_dir_all(directory).map_err(|error| write_error(&error))?;

        // A sibling temporary file keeps rename on the same filesystem. A failed write
        // leaves the active slot intact; replacing it never creates a missing-save gap.
        let temporary = directory.join(format!(".active-v1.{:016x}.tmp", fresh_seed()?));
        replace_file(&temporary, &self.path, data)
    }

    fn clear(&mut self) -> Result<(), StorageError> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(StorageError::Clear(error.to_string())),
        }
    }
}

fn replace_file(temporary: &Path, destination: &Path, data: &str) -> Result<(), StorageError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(temporary)
        .map_err(|error| write_error(&error))?;
    let result = (|| {
        file.write_all(data.as_bytes())
            .map_err(|error| write_error(&error))?;
        file.sync_all().map_err(|error| write_error(&error))?;
        // Closing the file before rename also permits replacement on Windows.
        drop(file);
        fs::rename(temporary, destination).map_err(|error| write_error(&error))
    })();
    if result.is_err() {
        // Cleanup only after successfully creating our own temporary file. Preserve
        // the original failure if cleanup fails, and never remove a colliding file.
        let _ = fs::remove_file(temporary);
    }
    result
}

fn write_error(error: &io::Error) -> StorageError {
    if error.kind() == io::ErrorKind::StorageFull {
        StorageError::Quota
    } else {
        StorageError::Write(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = env::temp_dir().join(format!(
                "low-kingdoms-storage-tests-{:016x}",
                fresh_seed().expect("test entropy")
            ));
            fs::create_dir(&path).expect("create isolated test directory");
            Self(path)
        }

        fn store(&self) -> FileStore {
            FileStore {
                path: self.0.join("low-kingdoms/active-v1.json"),
            }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn file_slot_round_trip_replacement_and_idempotent_clear() {
        let directory = TestDirectory::new();
        let mut store = directory.store();
        assert_eq!(store.read().expect("missing slot"), None);
        store.clear().expect("clear missing slot");
        store.write("first save").expect("create save");
        assert_eq!(store.read().expect("read save"), Some("first save".into()));
        store.write("short").expect("replace save");
        assert_eq!(
            store.read().expect("read replacement"),
            Some("short".into())
        );
        assert_eq!(
            fs::read_dir(store.path.parent().expect("save parent"))
                .expect("read save directory")
                .count(),
            1,
            "successful writes leave only the active slot"
        );
        store.clear().expect("clear save");
        store.clear().expect("clear missing save again");
        assert_eq!(store.read().expect("cleared slot"), None);
    }

    #[test]
    fn storage_returns_text_without_interpreting_envelope_corruption() {
        let directory = TestDirectory::new();
        let mut store = directory.store();
        store.write("invalid envelope {").expect("write raw text");
        assert_eq!(
            store.read().expect("session validates text"),
            Some("invalid envelope {".into())
        );
        fs::write(&store.path, [0xff, 0xfe]).expect("write invalid UTF-8");
        assert_eq!(store.read(), Err(StorageError::Encoding));
        assert_eq!(
            fs::read(&store.path).expect("damaged bytes remain intact"),
            [0xff, 0xfe]
        );
    }

    #[test]
    fn damaged_encoding_requires_confirmation_then_starts_a_saved_battle() {
        use bevy::prelude::State;
        use low_kingdoms_battle::{content::Content, session::Session};

        use crate::{
            gameplay::{Activate, Battle, Panel, tests::application_with_store},
            screens::Screen,
            ui::Control,
        };

        let directory = TestDirectory::new();
        let mut store = directory.store();
        store.write("previous save").expect("create slot directory");
        let path = store.path.clone();
        let original = [0xff, 0xfe, 0x80];
        fs::write(&path, original).expect("binary-corrupt file fixture");
        assert_eq!(store.read(), Err(StorageError::Encoding));

        let mut app = application_with_store(Box::new(store));
        let battle = app.world().resource::<Battle>();
        assert!(!battle.resume_available);
        assert!(battle.error.is_none());
        assert!(battle.notification.contains("damaged text encoding"));
        assert_eq!(fs::read(&path).expect("original slot"), original);

        for control in [Control::Play, Control::Cancel, Control::Play] {
            app.world_mut().write_message(Activate(control));
            app.update();
            app.update();
            assert_eq!(
                fs::read(&path).expect("confirmation preserves slot"),
                original
            );
            assert_eq!(
                app.world().resource::<Battle>().panel,
                if control == Control::Cancel {
                    Panel::None
                } else {
                    Panel::Replace
                }
            );
            assert_eq!(
                *app.world().resource::<State<Screen>>().get(),
                Screen::Title
            );
        }

        app.world_mut()
            .write_message(Activate(Control::ConfirmReplace));
        app.update();
        app.update();
        assert_eq!(
            *app.world().resource::<State<Screen>>().get(),
            Screen::Mulligan
        );
        assert!(app.world().resource::<Battle>().error.is_none());
        let envelope: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&path).expect("confirmed replacement is valid text"),
        )
        .expect("valid local save envelope");
        let encoded = envelope["session"]
            .as_str()
            .expect("session in local envelope");
        let session = Session::restore(&Content::load().expect("content"), encoded)
            .expect("confirmed new battle is resumable");
        assert_eq!(
            session.view().phase,
            low_kingdoms_rules::model::Phase::Mulligan
        );
    }

    #[test]
    fn unavailable_parent_and_non_file_slot_return_operation_errors() {
        let directory = TestDirectory::new();
        let blocked_parent = directory.0.join("not-a-directory");
        fs::write(&blocked_parent, "keep this file").expect("block parent directory");
        let mut store = FileStore {
            path: blocked_parent.join("active-v1.json"),
        };
        assert!(matches!(store.write("save"), Err(StorageError::Write(_))));
        assert_eq!(
            fs::read_to_string(&blocked_parent).expect("existing file intact"),
            "keep this file"
        );
        let mut store = FileStore {
            path: directory.0.clone(),
        };
        assert!(matches!(store.read(), Err(StorageError::Read(_))));
        assert!(matches!(store.clear(), Err(StorageError::Clear(_))));
    }

    #[test]
    fn failed_replacement_preserves_previous_destination() {
        let directory = TestDirectory::new();
        let temporary = directory.0.join("temporary");
        let destination = directory.0.join("existing-directory");
        fs::create_dir(&destination).expect("create replacement blocker");
        fs::write(destination.join("previous"), "valid previous save")
            .expect("write preserved fixture");
        assert!(matches!(
            replace_file(&temporary, &destination, "replacement"),
            Err(StorageError::Write(_))
        ));
        assert_eq!(
            fs::read_to_string(destination.join("previous")).expect("destination intact"),
            "valid previous save"
        );
        assert!(
            !temporary.exists(),
            "failed replacement cleans up its temporary file"
        );
    }

    #[test]
    fn colliding_temporary_file_is_not_overwritten_or_removed() {
        let directory = TestDirectory::new();
        let temporary = directory.0.join("temporary");
        let destination = directory.0.join("active-v1.json");
        fs::write(&temporary, "another pending write").expect("temporary collision fixture");
        fs::write(&destination, "previous save").expect("previous save fixture");
        assert!(matches!(
            replace_file(&temporary, &destination, "replacement"),
            Err(StorageError::Write(_))
        ));
        assert_eq!(
            fs::read_to_string(&temporary).expect("colliding temporary intact"),
            "another pending write"
        );
        assert_eq!(
            fs::read_to_string(&destination).expect("previous save intact"),
            "previous save"
        );
    }

    #[test]
    fn platform_paths_are_absolute_user_data_locations() {
        let home = env::temp_dir().join("test-home");
        let custom = env::temp_dir().join("custom-data");
        assert_eq!(
            app_data_directory("windows", None, Some(custom.clone()), None)
                .expect("Windows local app data"),
            custom.join("low-kingdoms")
        );
        assert_eq!(
            app_data_directory("macos", Some(home.clone()), None, None).expect("macOS user data"),
            home.join("Library/Application Support/low-kingdoms")
        );
        assert_eq!(
            app_data_directory("linux", None, None, Some(custom.clone())).expect("XDG user data"),
            custom.join("low-kingdoms")
        );
        for xdg in [None, Some(PathBuf::from("relative-data"))] {
            assert_eq!(
                app_data_directory("linux", Some(home.clone()), None, xdg).expect("XDG fallback"),
                home.join(".local/share/low-kingdoms")
            );
        }
        assert!(app_data_directory("windows", Some(home), None, None).is_err());
        assert!(app_data_directory("macos", None, None, None).is_err());
        assert!(app_data_directory("linux", Some(PathBuf::from("relative")), None, None).is_err());
        assert!(app_data_directory("linux", None, None, None).is_err());
    }

    #[cfg(feature = "test-support")]
    #[test]
    fn smoke_fixture_directory_requires_an_explicit_absolute_path() {
        let absolute = env::temp_dir().join("isolated-native-smoke");
        assert_eq!(
            test_directory(absolute.clone()).expect("absolute fixture path"),
            absolute
        );
        assert!(test_directory(PathBuf::from("relative-fixture")).is_err());
    }
}
