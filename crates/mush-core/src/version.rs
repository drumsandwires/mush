use shadow_rs::shadow;

shadow!(build);

/// Format a short version string for a mush shell binary.
///
/// # Example
/// ```
/// let s = mush_core::version::short("mish");
/// assert_eq!(s, "mish 0.1.0");
/// ```
pub fn short(name: &str) -> String {
    format!("{name} {}", build::PKG_VERSION)
}

/// Format a long version banner for a mush shell binary.
///
/// # Example
/// ```
/// let s = mush_core::version::long("mish");
/// ```
pub fn long(name: &str) -> String {
    format!(
        "{name} {}\n\
         branch:   {}\n\
         commit:   {}\n\
         built:    {}\n\
         rustc:    {}",
        build::PKG_VERSION,
        build::BRANCH,
        build::SHORT_COMMIT,
        build::BUILD_TIME,
        build::RUST_VERSION,
    )
}
