/// Format a short version string for a mush shell binary.
pub fn short(name: &str, pkg_version: &str) -> String {
    format!("{name} {pkg_version}")
}

/// Format a long version banner for a mush shell binary.
pub fn long(
    name: &str,
    pkg_version: &str,
    branch: &str,
    short_commit: &str,
    build_time: &str,
    rust_version: &str,
) -> String {
    format!(
        "{name} {pkg_version}\n\
         branch:   {branch}\n\
         commit:   {short_commit}\n\
         built:    {build_time}\n\
         rustc:    {rust_version}",
    )
}
