/// Format a short version string for a mush shell binary.
pub fn short(name: &str, pkg_version: &str) -> String {
    format!("{name} {pkg_version}")
}

/// Compact single-line form for log headers and status displays.
/// In debug builds, appends the short commit hash and dirty flag.
pub fn compact(name: &str, pkg_version: &str, short_commit: &str, clean: bool) -> String {
    let dirty = if clean { "" } else { "+dirty" };
    format!("{name} {pkg_version}+{short_commit}{dirty}")
}

/// Format a long version banner for a mush shell binary.
pub fn long(
    name: &str,
    pkg_version: &str,
    branch: &str,
    short_commit: &str,
    commit_date: &str,
    build_time: &str,
    rust_version: &str,
    rust_channel: &str,
    clean: bool,
) -> String {
    let dirty = if clean { "" } else { " (dirty)" };
    format!(
        "{name} {pkg_version}\n\
         branch:   {branch}\n\
         commit:   {short_commit}{dirty} {commit_date}\n\
         built:    {build_time}\n\
         rustc:    {rust_version} ({rust_channel})",
    )
}

/// Generates `short()`, `compact()`, `banner()`, and `long()` version
/// functions for a mush shell or plugin binary.
///
/// Each calling crate must have `shadow-rs` in both `[dependencies]`
/// and `[build-dependencies]`, and a `build.rs` that calls
/// `ShadowBuilder::builder().build().unwrap()`.
///
/// # Example
/// ```rust,ignore
/// mush_core::version_module!("mush");
/// ```
#[macro_export]
macro_rules! version_module {
    ($name:literal) => {
        use shadow_rs::shadow;
        shadow!(build);

        pub fn short() -> String {
            mush_core::version::short($name, build::PKG_VERSION)
        }

        pub fn compact() -> String {
            mush_core::version::compact(
                $name,
                build::PKG_VERSION,
                build::SHORT_COMMIT,
                build::GIT_CLEAN,
            )
        }

        pub fn banner() -> String {
            if cfg!(debug_assertions) {
                compact()
            } else {
                short()
            }
        }

        pub fn long() -> String {
            mush_core::version::long(
                $name,
                build::PKG_VERSION,
                build::BRANCH,
                build::SHORT_COMMIT,
                build::COMMIT_DATE,
                build::BUILD_TIME,
                build::RUST_VERSION,
                build::BUILD_RUST_CHANNEL,
                build::GIT_CLEAN,
            )
        }
    };
}