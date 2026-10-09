pub mod audio {
    pub mod display;
    pub mod engine;
    pub mod enumerate;
    pub mod types;
}

// Re-export only what lib.rs needs.
// pub use types::{AudioDeviceError, AudioSystemInfo};
// pub use enumerate::enumerate_hosts_and_devices;
// pub use display::list_hosts_and_devices;
