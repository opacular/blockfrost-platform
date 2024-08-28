use crate::errors::BlockfrostError;
use axum::http::HeaderMap;
use reqwest::header::CONTENT_TYPE;

/// Helper to validate content type or return custom BlockfrostError 400
pub fn validate_content_type(
    headers: &HeaderMap,
    allowed_headers: &[&str],
) -> Result<bool, BlockfrostError> {
    if let Some(content_type) = headers.get(CONTENT_TYPE) {
        if !allowed_headers.iter().any(|&header| header == content_type) {
            if allowed_headers.len() == 1 {
                return Err(BlockfrostError::custom_400(format!(
                    "Content-Type must be: {:?}",
                    allowed_headers[0]
                )));
            }

            return Err(BlockfrostError::custom_400(
                format!("Content-Type must be one of: {:?}", allowed_headers).to_string(),
            ));
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    #[rstest]
    #[case(&["application/json"], "application/json", true, None)]
    #[case(&["application/json", "application/xml"], "application/json", true, None)]
    #[case(&["application/json"], "application/xml", false, Some("BlockfrostError: Content-Type must be: \"application/json\""))]
    #[case(&["application/json", "application/xml"], "text/html", false, Some("BlockfrostError: Content-Type must be one of: [\"application/json\", \"application/xml\"]"))]
    #[case(&["application/json"], "", true, None)]
    #[case(&[], "application/json", false, Some("BlockfrostError: Content-Type must be one of: []"))]
    fn test_validate_content_type(
        #[case] allowed_headers: &[&str],
        #[case] content_type: &str,
        #[case] expected_ok: bool,
        #[case] expected_err: Option<&str>,
    ) {
        use axum::http::HeaderValue;
        let mut headers = HeaderMap::new();

        if !content_type.is_empty() {
            headers.insert(CONTENT_TYPE, HeaderValue::from_str(content_type).unwrap());
        }

        let result = validate_content_type(&headers, allowed_headers);

        if expected_ok {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
            if let Some(expected_err_msg) = expected_err {
                if let Err(e) = result {
                    assert_eq!(e.to_string(), expected_err_msg);
                }
            }
        }
    }
}
