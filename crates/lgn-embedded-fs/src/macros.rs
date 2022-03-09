#[macro_export]
macro_rules! embedded_watched_file {
    ( $name:ident, $file_path:literal ) => {
        pub static $name: $crate::EmbeddedFile = $crate::EmbeddedFile::new(
            concat!("crate://", env!("CARGO_PKG_NAME"), "/", $file_path),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $file_path)),
            Some(concat!(env!("CARGO_MANIFEST_DIR"), "/", $file_path)),
        );
    };
}

#[macro_export]
macro_rules! embedded_file {
    ( $name:ident, $file_path:literal ) => {
        pub static $name: $crate::EmbeddedFile = $crate::EmbeddedFile::new(
            concat!("crate://", env!("CARGO_PKG_NAME"), "/", $file_path),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $file_path)),
            None,
        );
    };
}
