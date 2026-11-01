use shadow_rs::shadow;

shadow!(build);

pub fn short() -> String {
    mush_core::version::short("mash", build::PKG_VERSION)
}

pub fn long() -> String {
    mush_core::version::long(
        "mash",
        build::PKG_VERSION,
        build::BRANCH,
        build::SHORT_COMMIT,
        build::BUILD_TIME,
        build::RUST_VERSION,
    )
}