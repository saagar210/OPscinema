#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn CGPreflightScreenCaptureAccess() -> bool;
}

pub fn screen_recording_enabled() -> bool {
    #[cfg(target_os = "macos")]
    // SAFETY: CoreGraphics function has no arguments and no side effects beyond permission check.
    unsafe {
        CGPreflightScreenCaptureAccess()
    }

    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

pub fn accessibility_enabled() -> bool {
    #[cfg(target_os = "macos")]
    // SAFETY: ApplicationServices function has no arguments and no side effects beyond permission check.
    unsafe {
        AXIsProcessTrusted()
    }

    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

pub fn full_disk_access_enabled() -> bool {
    // macOS has no stable direct API for FDA status; treat as unknown/false unless validated elsewhere.
    false
}
