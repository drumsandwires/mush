/// Attempt to elevate the current thread to real-time priority.
/// Fails silently — the system remains functional at normal priority.
/// Call once on entry to the audio callback and MIDI RT thread.
pub fn elevate_current_thread() {
    #[cfg(target_os = "linux")]
    linux::elevate();

    #[cfg(target_os = "macos")]
    macos::elevate();

    // Windows and other platforms — no-op at MVP.
}

#[cfg(target_os = "linux")]
mod linux {
    pub fn elevate() {
        unsafe {
            let param = libc::sched_param { sched_priority: 10 };
            libc::sched_setscheduler(0, libc::SCHED_FIFO, &param);
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    pub fn elevate() {
        unsafe {
            libc::pthread_setschedprio(libc::pthread_self(), 47);
        }
    }
}

// TODO: RT elevate for Windows?
