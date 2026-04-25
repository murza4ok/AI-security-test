use anyhow::{bail, Result};

#[derive(Debug, Clone)]
pub struct HttpTargetConfig {
    pub base_url: String,
    pub username: String,
    pub profile: String,
}

pub fn build_http_target_config(
    target_mode: Option<&str>,
    base_url: Option<&str>,
    username: Option<&str>,
    profile: Option<&str>,
) -> Result<Option<HttpTargetConfig>> {
    match target_mode {
        None => match (base_url, username, profile) {
            (None, None, None) => Ok(None),
            _ => bail!(
                "--target-mode http is required when using --target-base-url, --target-user, or --target-profile"
            ),
        },
        Some("http") => match (base_url, username, profile) {
            (Some(base_url), Some(username), Some(profile)) => Ok(Some(HttpTargetConfig {
                base_url: base_url.trim_end_matches('/').to_string(),
                username: username.to_string(),
                profile: profile.to_string(),
            })),
            _ => bail!(
                "--target-base-url, --target-user, and --target-profile must be provided together for --target-mode http"
            ),
        },
        Some(other) => bail!("unsupported target mode: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_none_when_http_mode_not_selected() {
        let config = build_http_target_config(None, None, None, None).unwrap();
        assert!(config.is_none());
    }

    #[test]
    fn requires_explicit_http_mode_for_target_flags() {
        let error = build_http_target_config(
            None,
            Some("http://127.0.0.1:3000"),
            Some("customer_alice"),
            Some("naive"),
        )
        .err()
        .expect("expected missing target mode to fail");

        assert!(error.to_string().contains("--target-mode http is required"));
    }

    #[test]
    fn builds_http_target_config() {
        let config = build_http_target_config(
            Some("http"),
            Some("http://127.0.0.1:3000/"),
            Some("customer_alice"),
            Some("naive"),
        )
        .unwrap()
        .expect("expected http config");

        assert_eq!(config.base_url, "http://127.0.0.1:3000");
        assert_eq!(config.username, "customer_alice");
        assert_eq!(config.profile, "naive");
    }
}
