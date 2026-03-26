//! Unit tests for [`crate::protocols::proxy`] (URL encoding, HTML rewrite, adblock hooks).

use std::sync::Arc;

use adblock::{lists::ParseOptions, Engine, FilterSet};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use tauri::Url;

use crate::protocols::proxy::{
    self, CELADON_SCHEME_ORIGIN, RewriteContext,
};

#[test]
fn article_token_round_trip_with_query_and_fragment() {
    let url = "https://example.com/path?q=1&x=y#frag";
    let tok = proxy::encode_article_url_token(url);
    let decoded = proxy::decode_asset_token(&tok).expect("decode");
    assert_eq!(decoded, url);
}

#[test]
fn asset_payload_round_trip() {
    let src = "https://page.com/doc";
    let res = "https://cdn.com/x.png";
    let tok = proxy::encode_asset_payload(src, res);
    let (s, r) = proxy::decode_asset_payload(&tok).expect("decode payload");
    assert_eq!(s, src);
    assert_eq!(r, res);
}

#[test]
fn asset_payload_fallback_no_tab() {
    let url = "https://only-resource.com/z.js";
    let tok = encode_asset_token_only(url);
    let (s, r) = proxy::decode_asset_payload(&tok).unwrap();
    assert!(s.is_empty());
    assert_eq!(r, url);
}

fn encode_asset_token_only(resource_url: &str) -> String {
    URL_SAFE_NO_PAD.encode(resource_url.as_bytes())
}

#[test]
fn asset_payload_rejects_empty_tab_sides() {
    let bad = URL_SAFE_NO_PAD.encode("\thttps://a.com".as_bytes());
    assert!(proxy::decode_asset_payload(&bad).is_err());
    let bad2 = URL_SAFE_NO_PAD.encode("https://a.com\t".as_bytes());
    assert!(proxy::decode_asset_payload(&bad2).is_err());
}

#[test]
fn resolve_against_base_cases() {
    let base = Url::parse("https://example.com/dir/page.html").unwrap();
    assert_eq!(
        proxy::resolve_against_base(&base, "/abs")
            .unwrap()
            .as_str(),
        "https://example.com/abs"
    );
    assert_eq!(
        proxy::resolve_against_base(&base, "rel.js")
            .unwrap()
            .as_str(),
        "https://example.com/dir/rel.js"
    );
    assert_eq!(
        proxy::resolve_against_base(&base, "//cdn.net/x")
            .unwrap()
            .as_str(),
        "https://cdn.net/x"
    );
    assert!(proxy::resolve_against_base(&base, "").is_none());
}

#[test]
fn parse_article_document_url_cases() {
    let url = "https://news.test/article";
    let tok = proxy::encode_article_url_token(url);
    let path = format!("/article/{tok}");
    assert_eq!(proxy::parse_article_document_url(&path).unwrap(), url);
    assert!(proxy::parse_article_document_url("/feed/x").is_err());
    assert!(proxy::parse_article_document_url("/article/").is_err());
}

#[test]
fn build_article_proxy_url_format() {
    let u = "https://a.com/b";
    let full = proxy::build_article_proxy_url(u);
    assert!(full.starts_with(CELADON_SCHEME_ORIGIN));
    let path = full.strip_prefix(CELADON_SCHEME_ORIGIN).unwrap();
    assert!(path.starts_with("/article/"));
    assert_eq!(proxy::parse_article_document_url(path).unwrap(), u);
}

#[test]
fn network_blocked_matches_easy_rule() {
    let mut fs = FilterSet::new(false);
    fs.add_filters(&["||bad.example^".to_owned()], ParseOptions::default());
    let engine = Engine::from_filter_set(fs, true);
    let src = "https://page.com/";
    assert!(proxy::network_blocked(
        &engine,
        "https://bad.example/track.js",
        src,
        "script"
    ));
    assert!(!proxy::network_blocked(
        &engine,
        "https://ok.example/ok.js",
        src,
        "script"
    ));
}

#[test]
fn rewrite_html_strips_csp_meta() {
    let html = r#"<!doctype html><html><head>
            <meta http-equiv="Content-Security-Policy" content="default-src 'none'; frame-ancestors 'none'">
            <title>t</title></head><body><p>hi</p></body></html>"#;
    let engine = proxy::build_filter_engine_for_app();
    let base = Url::parse("https://example.com/").unwrap();
    let ctx = Arc::new(RewriteContext {
        engine,
        source_url: "https://example.com/".to_string(),
        base_url: base,
    });
    let out = proxy::rewrite_html(html, ctx).expect("rewrite");
    assert!(!out.to_ascii_lowercase().contains("content-security-policy"));
}
