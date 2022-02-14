macro_rules! assert_content_not_found {
    ($provider:expr, $id:expr) => {{
        match $provider.read_content(&$id).await {
            Ok(_) => panic!("content was found with the specified identifier `{}`", $id),
            Err(Error::NotFound {}) => {}
            Err(err) => panic!("unexpected error: {}", err),
        };
    }};
}

macro_rules! assert_read_content {
    ($provider:expr, $id:expr, $expected_content:expr) => {{
        let content = $provider
            .read_content(&$id)
            .await
            .expect("failed to read content");

        assert_eq!(
            $expected_content,
            String::from_utf8(content).expect("failed to parse content")
        );
    }};
}

macro_rules! assert_write_content {
    ($provider:expr, $content:expr) => {{
        #[allow(clippy::string_lit_as_bytes)]
        $provider
            .write_content($content.as_bytes())
            .await
            .expect("failed to write content")
    }};
}

pub(crate) use {assert_content_not_found, assert_read_content, assert_write_content};
