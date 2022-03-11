macro_rules! assert_alias_not_found {
    ($provider:expr, $key_space:expr, $key:expr) => {{
        match $provider.read_alias($key_space, $key).await {
            Ok(_) => panic!(
                "alias was found with the specified key `{}/{}`",
                $key_space, $key
            ),
            Err(Error::NotFound {}) => {}
            Err(err) => panic!("unexpected error: {}", err),
        };
    }};
}

macro_rules! assert_content_not_found {
    ($provider:expr, $id:expr) => {{
        match $provider.read_content(&$id).await {
            Ok(_) => panic!("content was found with the specified identifier `{}`", $id),
            Err(Error::NotFound {}) => {}
            Err(err) => panic!("unexpected error: {}", err),
        };
    }};
}

macro_rules! assert_read_alias {
    ($provider:expr, $key_space:expr, $key:expr, $expected_content:expr) => {{
        let content = $provider
            .read_alias($key_space, $key)
            .await
            .expect("failed to read alias");

        assert_eq!(content, $expected_content);
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
        let expected_contents: Vec<_> = $expected_contents.into_iter().collect();
        let ids: Vec<_> = $ids.into_iter().collect();

        assert_eq!(expected_contents.len(), ids.len());

        let expected_contents = ids
            .iter()
            .cloned()
            .zip(expected_contents)
            .collect::<BTreeMap<_, _>>();
        let ids = ids.into_iter().collect::<BTreeSet<Identifier>>();

        let contents = $provider
            .read_contents(&ids)
            .await
            .expect("failed to read contents");

        for id in &ids {
            let content = contents
                .get(id)
                .expect(&format!("failed to find content for `{}`", id));
            let expected_content = expected_contents
                .get(id)
                .expect(&format!("failed to find expected content for `{}`", id));

            match (content, expected_content) {
                (Ok(content), Ok(expected_content)) => assert_eq!(content, expected_content),
                (Err(err), Err(expected_err)) => {
                    assert_eq!(err.to_string(), expected_err.to_string())
                }
                (Ok(_), Err(_)) => {
                    panic!("content was found with the specified identifier `{}`", id)
                }
                (Err(_), Ok(_)) => panic!(
                    "content was not found with the specified identifier `{}`",
                    id
                ),
            }
        }
    }};
}

macro_rules! assert_write_alias {
    ($provider:expr, $key_space:expr, $key:expr, $content:expr) => {{
        #[allow(clippy::string_lit_as_bytes)]
        $provider
            .write_alias($key_space, $key, $content)
            .await
            .expect("failed to write alias")
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
    assert_alias_not_found, assert_content_not_found, assert_read_alias, assert_read_content,
    assert_read_contents, assert_write_alias, assert_write_avoided, assert_write_content,
};
