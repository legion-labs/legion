/// A resource for configuring usage of the `rust_tao` library.
#[derive(Debug, Default)]
pub struct TaoConfig {
    /// Configures the tao library to return control to the main thread after
    /// the [run](legion_app::App::run) loop is exited. Tao strongly recommends
    /// avoiding this when possible. Before using this please read and understand
    /// the following:
    ///
    /// # Caveats
    /// return from run, despite its appearance at first glance, this is *not* a perfect replacement for
    /// `poll_events`. For example, this function will not return on Windows or macOS while a
    /// window is getting resized, resulting in all application logic outside of the
    /// `event_handler` closure not running until the resize operation ends. Other OS operations
    /// may also result in such freezes. This behavior is caused by fundamental limitations in the
    /// underlying OS APIs, which cannot be hidden by `tao` without severe stability repercussions.
    ///
    /// You are strongly encouraged to use `run`, unless the use of this is absolutely necessary.
    ///
    /// This feature is only available on desktop `target_os` configurations.
    /// Namely `windows`, `macos`, `linux`, `dragonfly`, `freebsd`, `netbsd`, and
    /// `openbsd`. If set to true on an unsupported platform
    /// [run](legion_app::App::run) will panic.
    pub return_from_run: bool,
}
