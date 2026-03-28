use std::sync::Arc;

use adblock::{request::Request as BlockRequest, Engine, lists::ParseOptions, FilterSet};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use http::header::CONTENT_TYPE;
use http::{Request, Response, StatusCode};
use lol_html::{element, rewrite_str, RewriteStrSettings};
use tauri::{Manager, Runtime, UriSchemeContext, UriSchemeResponder};
use tauri::Url;

use crate::syndication::syndicator::{http_client, FetchError};

pub const CELADON_SCHEME_ORIGIN: &str = "celadon://localhost";

pub fn build_article_proxy_url(document_url: &str) -> String {
    format!(
        "{CELADON_SCHEME_ORIGIN}/article/{}",
        encode_article_url_token(document_url)
    )
}

/// Shared across lol_html element handlers: cheap [`Arc`] clones, safe for concurrent read-only use.
pub(crate) struct RewriteContext {
    pub(crate) engine: Arc<Engine>,
    pub(crate) source_url: String,
    pub(crate) base_url: Url,
}

fn build_filter_engine() -> Arc<Engine> {
    let mut filter_set = FilterSet::new(false);
    filter_set.add_filters(
        &[
            include_str!("blocklists/easylist.txt"),
            include_str!("blocklists/privacy.txt"),
        ],
        ParseOptions::default(),
    );
    Arc::new(Engine::from_filter_set(filter_set, true))
}

/// Managed application state (see `main`: `app.manage(build_filter_engine_for_app())`).
pub fn build_filter_engine_for_app() -> Arc<Engine> {
    build_filter_engine()
}

/// Base64url token for `…/article/{token}` (encode the raw HTTPS/HTML document URL).
pub fn encode_article_url_token(document_url: &str) -> String {
    URL_SAFE_NO_PAD.encode(document_url.as_bytes())
}

/// So the dev shell can `fetch(celadon://…)` and use `srcdoc` (iframe navigation to custom schemes is often blocked under https).
const CORS_ALLOW_ANY: &str = "Access-Control-Allow-Origin";

fn error_response(status: StatusCode, message: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(CORS_ALLOW_ANY, "*")
        .body(message.as_bytes().to_vec())
        .expect("infallible error response")
}

const ASSET_PAYLOAD_SEP: char = '\t';

// Encodes URL to simplify parsing
pub(crate) fn encode_asset_payload(source_url: &str, resource_url: &str) -> String {
    let payload = format!("{source_url}{ASSET_PAYLOAD_SEP}{resource_url}");
    URL_SAFE_NO_PAD.encode(payload.as_bytes())
}

pub(crate) fn decode_asset_token(token: &str) -> Result<String, &'static str> {
    let bytes = URL_SAFE_NO_PAD
        .decode(token.as_bytes())
        .map_err(|_| "invalid asset token")?;
    String::from_utf8(bytes).map_err(|_| "asset token is not utf-8")
}

/// Decodes tab-separated `(source_url, resource_url)`; falls back to `(empty, whole)` for bare URLs.
pub(crate) fn decode_asset_payload(token: &str) -> Result<(String, String), &'static str> {
    let s = decode_asset_token(token)?;
    if let Some((src, res)) = s.split_once(ASSET_PAYLOAD_SEP) {
        if src.is_empty() || res.is_empty() {
            return Err("invalid asset payload");
        }
        return Ok((src.to_string(), res.to_string()));
    }
    if s.is_empty() {
        return Err("empty asset payload");
    }
    Ok((String::new(), s))
}

/// Proxied subresource URL embedded in rewritten HTML.
fn proxied_asset_url(source_url: &str, remote: &Url) -> String {
    let token = encode_asset_payload(source_url, remote.as_str());
    format!("{CELADON_SCHEME_ORIGIN}/asset/{token}")
}

fn infer_request_type(url: &Url) -> &'static str {
    let path = url.path().to_ascii_lowercase();
    if path.ends_with(".css") {
        return "stylesheet";
    }
    if path.ends_with(".js") || path.ends_with(".mjs") || path.ends_with(".cjs") {
        return "script";
    }
    if path.ends_with(".png")
        || path.ends_with(".jpg")
        || path.ends_with(".jpeg")
        || path.ends_with(".gif")
        || path.ends_with(".webp")
        || path.ends_with(".svg")
        || path.ends_with(".ico")
        || path.ends_with(".avif")
        || path.ends_with(".jxl")
    {
        return "image";
    }
    if path.ends_with(".woff")
        || path.ends_with(".woff2")
        || path.ends_with(".ttf")
        || path.ends_with(".otf")
    {
        return "font";
    }
    "other"
}

pub(crate) fn resolve_against_base(base: &Url, reference: &str) -> Option<Url> {
    let reference = reference.trim();
    if reference.is_empty() {
        return None;
    }
    if reference.starts_with("//") {
        let scheme = base.scheme();
        Url::parse(&format!("{scheme}:{reference}")).ok()
    } else {
        base.join(reference).ok()
    }
}

pub(crate) fn network_blocked(
    engine: &Engine,
    resource_url: &str,
    source_url: &str,
    request_type: &str,
) -> bool {
    match BlockRequest::new(resource_url, source_url, request_type) {
        Ok(req) => engine.check_network_request(&req).matched,
        Err(_) => false,
    }
}

pub(crate) fn rewrite_html(
    html: &str,
    ctx: Arc<RewriteContext>,
) -> Result<String, lol_html::errors::RewritingError> {
    let element_content_handlers = vec![
        // Drop CSP meta tags that can block embedding (e.g. frame-ancestors) inside the proxied document.
        element!("meta", |el| {
            let he = el
                .get_attribute("http-equiv")
                .unwrap_or_default()
                .to_ascii_lowercase();
            if he == "content-security-policy" {
                el.remove();
            }
            Ok(())
        }),
        element!("img", {
            let ctx = Arc::clone(&ctx);
            move |el| {
                if let Some(src) = el.get_attribute("src") {
                    if src.starts_with("data:") || src.starts_with("blob:") {
                        return Ok(());
                    }
                    if let Some(abs) = resolve_against_base(&ctx.base_url, &src) {
                        if network_blocked(&ctx.engine, abs.as_str(), &ctx.source_url, "image") {
                            el.remove();
                        } else {
                            el.set_attribute("src", &proxied_asset_url(&ctx.source_url, &abs))?;
                        }
                    }
                }
                Ok(())
            }
        }),
        element!("iframe", {
            let ctx = Arc::clone(&ctx);
            move |el| {
                if let Some(src) = el.get_attribute("src") {
                    if src.starts_with("data:") || src.starts_with("blob:") {
                        return Ok(());
                    }
                    if let Some(abs) = resolve_against_base(&ctx.base_url, &src) {
                        if network_blocked(&ctx.engine, abs.as_str(), &ctx.source_url, "sub_frame") {
                            el.remove();
                        } else {
                            el.set_attribute("src", &proxied_asset_url(&ctx.source_url, &abs))?;
                        }
                    }
                }
                Ok(())
            }
        }),
        element!("script", {
            let ctx = Arc::clone(&ctx);
            move |el| {
                if let Some(src) = el.get_attribute("src") {
                    if src.starts_with("data:") || src.starts_with("blob:") {
                        return Ok(());
                    }
                    if let Some(abs) = resolve_against_base(&ctx.base_url, &src) {
                        if network_blocked(&ctx.engine, abs.as_str(), &ctx.source_url, "script") {
                            el.remove();
                        } else {
                            el.set_attribute("src", &proxied_asset_url(&ctx.source_url, &abs))?;
                        }
                    }
                }
                Ok(())
            }
        }),
        element!("link", {
            let ctx = Arc::clone(&ctx);
            move |el| {
                let rel = el
                    .get_attribute("rel")
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                if !rel.split_whitespace().any(|t| t == "stylesheet") {
                    return Ok(());
                }
                if let Some(href) = el.get_attribute("href") {
                    if href.starts_with("data:") || href.starts_with("blob:") {
                        return Ok(());
                    }
                    if let Some(abs) = resolve_against_base(&ctx.base_url, &href) {
                        if network_blocked(&ctx.engine, abs.as_str(), &ctx.source_url, "stylesheet") {
                            el.remove();
                        } else {
                            el.set_attribute("href", &proxied_asset_url(&ctx.source_url, &abs))?;
                        }
                    }
                }
                Ok(())
            }
        }),
    ];

    rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers,
            ..RewriteStrSettings::new()
        },
    )
}

pub(crate) fn parse_article_document_url(path: &str) -> Result<String, &'static str> {
    const PREFIX: &str = "/article/";
    if !path.starts_with(PREFIX) {
        return Err("path must start with /article/");
    }
    let token = path.trim_start_matches(PREFIX);
    if token.is_empty() {
        return Err("missing article token");
    }
    decode_asset_token(token)
}

async fn fetch_rewrite_article_response(document_url: &str, engine: Arc<Engine>) -> Result<Response<Vec<u8>>, FetchError> {
    let client = http_client().map_err(|e| FetchError::Network(Box::new(e)))?;
    let response = client
        .get(document_url)
        .send()
        .await
        .map_err(|e| FetchError::Network(Box::new(e)))?;
    let bytes = response
        .bytes()
        .await
        .map_err(|e| FetchError::Network(Box::new(e)))?;
    let html = String::from_utf8_lossy(&bytes).into_owned();

    let base_url = Url::parse(document_url).map_err(|e| FetchError::Network(Box::new(e)))?;
    let ctx = Arc::new(RewriteContext {
        engine,
        source_url: document_url.to_string(),
        base_url,
    });

    let rewritten = rewrite_html(&html, ctx).map_err(|e| {
        FetchError::Network(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        )))
    })?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html; charset=utf-8")
        .header(CORS_ALLOW_ANY, "*")
        .body(rewritten.into_bytes())
        .expect("infallible html response"))
}

pub fn handle_article<R: Runtime>(ctx: UriSchemeContext<'_, R>, req: Request<Vec<u8>>, responder: UriSchemeResponder) {
    let path = req.uri().path().to_string();
    let engine = match ctx.app_handle().try_state::<Arc<Engine>>() {
        Some(s) => s.inner().clone(),
        None => {
            responder.respond(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "adblock engine not initialized",
            ));
            return;
        }
    };

    let document_url = match parse_article_document_url(&path) {
        Ok(u) => u,
        Err(msg) => {
            responder.respond(error_response(StatusCode::BAD_REQUEST, msg));
            return;
        }
    };

    tauri::async_runtime::spawn(async move {
        let out = fetch_rewrite_article_response(&document_url, engine).await;
        match out {
            Ok(resp) => responder.respond(resp),
            Err(e) => responder.respond(error_response(
                StatusCode::BAD_GATEWAY,
                &e.to_string(),
            )),
        }
    });
}

/// Fetches a remote subresource identified by the `/asset/<token>` path.
pub fn handle_asset<R: Runtime>(ctx: UriSchemeContext<'_, R>, req: Request<Vec<u8>>, responder: UriSchemeResponder) {
    let path = req.uri().path().to_string();
    const PREFIX: &str = "/asset/";
    let token = if let Some(rest) = path.strip_prefix(PREFIX) {
        rest
    } else {
        responder.respond(error_response(
            StatusCode::BAD_REQUEST,
            "path must start with /asset/",
        ));
        return;
    };

    if token.is_empty() {
        responder.respond(error_response(StatusCode::BAD_REQUEST, "missing asset token"));
        return;
    }

    let (source_url, remote_url) = match decode_asset_payload(token) {
        Ok(p) => p,
        Err(msg) => {
            responder.respond(error_response(StatusCode::BAD_REQUEST, msg));
            return;
        }
    };

    let parsed = match Url::parse(&remote_url) {
        Ok(u) => u,
        Err(e) => {
            responder.respond(error_response(
                StatusCode::BAD_REQUEST,
                &format!("invalid URL: {e}"),
            ));
            return;
        }
    };

    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        responder.respond(error_response(
            StatusCode::BAD_REQUEST,
            "only http and https URLs may be proxied",
        ));
        return;
    }

    let engine = match ctx.app_handle().try_state::<Arc<Engine>>() {
        Some(s) => s.inner().clone(),
        None => {
            responder.respond(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "adblock engine not initialized",
            ));
            return;
        }
    };

    let request_type = infer_request_type(&parsed);

    tauri::async_runtime::spawn(async move {
        let blocked = match BlockRequest::new(&remote_url, &source_url, request_type) {
            Ok(req) => engine.check_network_request(&req).matched,
            Err(_) => false,
        };
        if blocked {
            responder.respond(
                Response::builder()
                    .status(StatusCode::NO_CONTENT)
                    .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                    .header(CORS_ALLOW_ANY, "*")
                    .body(Vec::new())
                    .expect("infallible blocked response"),
            );
            return;
        }

        let client = match http_client() {
            Ok(c) => c,
            Err(e) => {
                responder.respond(error_response(
                    StatusCode::BAD_GATEWAY,
                    &format!("http client: {e}"),
                ));
                return;
            }
        };

        let response = match client.get(remote_url.clone()).send().await {
            Ok(r) => r,
            Err(e) => {
                responder.respond(error_response(
                    StatusCode::BAD_GATEWAY,
                    &format!("fetch failed: {e}"),
                ));
                return;
            }
        };

        let status = StatusCode::from_u16(response.status().as_u16())
            .unwrap_or(StatusCode::BAD_GATEWAY);
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        let bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                responder.respond(error_response(
                    StatusCode::BAD_GATEWAY,
                    &format!("read body: {e}"),
                ));
                return;
            }
        };

        let mut builder = Response::builder().status(status);
        builder = builder.header(CONTENT_TYPE, content_type);
        builder = builder.header(CORS_ALLOW_ANY, "*");
        match builder.body(bytes.to_vec()) {
            Ok(resp) => responder.respond(resp),
            Err(e) => responder.respond(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("build response: {e}"),
            )),
        }
    });
}
