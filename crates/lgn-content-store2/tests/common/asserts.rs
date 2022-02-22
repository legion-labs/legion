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

        assert_eq!(content, $expected_content);
    }};
}

macro_rules! assert_read_contents {
    ($provider:expr, $ids:expr, $expected_contents:expr) => {{
        let contents = $provider
            .read_contents($ids)
            .await
            .expect("failed to read contents");

        assert_eq!(contents.len(), $expected_contents.len());

        for (i, content) in contents.iter().enumerate() {
            let expected_content = &$expected_contents[i];

            match (content, expected_content) {
                (Ok(content), Ok(expected_content)) => assert_eq!(content, expected_content),
                (Err(err), Err(expected_err)) => match (err, expected_err) {
                    (Error::NotFound { .. }, Error::NotFound { .. }) => {}
                    (err, expected_err) => {
                        panic!("unexpected errors: {:?} & {:?}", err, expected_err)
                    }
                },
                (Ok(_), Err(_)) => panic!("content was found at index {}", i),
                (Err(_), Ok(_)) => panic!("content was not found at index {}", i),
            };
        }
    }};
}

macro_rules! assert_write_content {
    ($provider:expr, $content:expr) => {{
        #[allow(clippy::string_lit_as_bytes)]
        $provider
            .write_content($content)
            .await
            .expect("failed to write content")
    }};
}

macro_rules! assert_write_avoided {
    ($provider:expr, $id:expr) => {{
        #[allow(clippy::string_lit_as_bytes)]
        match $provider.get_content_writer($id).await {
            Ok(_) => panic!(
                "content was written with the specified identifier `{}`",
                $id
            ),
            Err(Error::AlreadyExists {}) => {}
            Err(err) => panic!("unexpected error: {}", err),
        }
    }};
}

pub(crate) use {
    assert_content_not_found, assert_read_content, assert_read_contents, assert_write_avoided,
    assert_write_content,
};
