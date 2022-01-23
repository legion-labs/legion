#[macro_export]
macro_rules! embedded_watched_file {
    ( $name:ident, $file_path:literal ) => {
        #[linkme::distributed_slice($crate::EMBEDDED_FILES)]
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
        #[linkme::distributed_slice($crate::EMBEDDED_FILES)]
        pub static $name: $crate::EmbeddedFile = $crate::EmbeddedFile::new(
            concat!("crate://", env!("CARGO_PKG_NAME"), "/", $file_path),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $file_path)),
            None,
        );
    };
}
