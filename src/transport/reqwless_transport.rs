use crate::error::Error;
use crate::postgrest::{Method, PostgrestBuilder};
use crate::response::SupabaseResponse;
use embedded_nal_async::{Dns, TcpConnect};
use reqwless::client::HttpClient;
use reqwless::headers::ContentType;
use reqwless::request::Method as ReqwlessMethod;

fn to_reqwless_method(m: Method) -> ReqwlessMethod {
    match m {
        Method::Get => ReqwlessMethod::GET,
        Method::Post => ReqwlessMethod::POST,
        Method::Patch => ReqwlessMethod::PATCH,
        Method::Delete => ReqwlessMethod::DELETE,
    }
}

/// Execute a PostgrestBuilder request using a reqwless HttpClient.
///
/// Constructs the full URL, adds Supabase auth headers, sends the request,
/// and returns a SupabaseResponse referencing the rx_buf.
///
/// # Arguments
/// * `http` - A reqwless HttpClient with TCP + DNS
/// * `builder` - The configured PostgrestBuilder
/// * `url_buf` - Scratch buffer for constructing the full URL
/// * `rx_buf` - Buffer for the HTTP response body
pub async fn execute<'a, 'b, T, D, const N: usize>(
    http: &mut HttpClient<'a, T, D>,
    builder: &PostgrestBuilder<'b, N>,
    url_buf: &mut [u8],
    rx_buf: &'b mut [u8],
) -> Result<SupabaseResponse<'b>, Error>
where
    T: TcpConnect + 'a,
    D: Dns + 'a,
{
    let base = builder.base_url();
    let path = builder.path();
    let total_len = base.len() + path.len();

    if total_len > url_buf.len() {
        return Err(Error::UrlTooLong);
    }

    url_buf[..base.len()].copy_from_slice(base.as_bytes());
    url_buf[base.len()..total_len].copy_from_slice(path.as_bytes());
    let url = core::str::from_utf8(&url_buf[..total_len]).map_err(|_| Error::Utf8)?;

    let method = to_reqwless_method(builder.method());

    let mut request = http
        .request(method, url)
        .await
        .map_err(|_| Error::ConnectionFailed)?;

    if let Some(auth) = builder.auth_ref() {
        request = request.headers(&[("apikey", auth.api_key)]);
        if let Some(token) = auth.bearer_token {
            let mut bearer_val: heapless::String<512> = heapless::String::new();
            bearer_val
                .push_str("Bearer ")
                .map_err(|_| Error::BufferTooSmall)?;
            bearer_val
                .push_str(token)
                .map_err(|_| Error::BufferTooSmall)?;

            // We need to pass headers as a slice, but bearer_val is local.
            // reqwless headers() takes &[(&str, &str)] so we must chain calls.
            request = request.headers(&[("Authorization", bearer_val.as_str())]);
        }
    }

    // Build Prefer header value
    let mut prefer_parts: heapless::String<128> = heapless::String::new();
    let mut has_prefer = false;
    if builder.wants_upsert() {
        let _ = prefer_parts.push_str("resolution=merge-duplicates");
        has_prefer = true;
    }
    if builder.wants_return() {
        if has_prefer {
            let _ = prefer_parts.push_str(", ");
        }
        let _ = prefer_parts.push_str("return=representation");
        has_prefer = true;
    }
    if builder.wants_count() {
        if has_prefer {
            let _ = prefer_parts.push_str(", ");
        }
        let _ = prefer_parts.push_str("count=exact");
        has_prefer = true;
    }
    if has_prefer {
        request = request.headers(&[("Prefer", prefer_parts.as_str())]);
    }

    let response = if let Some(body_bytes) = builder.body() {
        request
            .body(body_bytes)
            .content_type(ContentType::ApplicationJson)
            .send(rx_buf)
            .await
            .map_err(|_| Error::ConnectionFailed)?
    } else {
        request
            .send(rx_buf)
            .await
            .map_err(|_| Error::ConnectionFailed)?
    };

    let status = response.status.0;
    let body = response
        .body()
        .read_to_end()
        .await
        .map_err(|_| Error::BufferTooSmall)?;

    Ok(SupabaseResponse::new(status, body))
}
