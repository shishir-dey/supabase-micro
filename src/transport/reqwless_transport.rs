use crate::error::Error;
use crate::postgrest::{Method, PostgrestBuilder};
use crate::response::SupabaseResponse;
use crate::transport::request::{write_url, RequestHeaders, MAX_REQUEST_HEADERS};
use embedded_nal_async::{Dns, TcpConnect};
use reqwless::client::HttpClient;
use reqwless::headers::ContentType;
use reqwless::request::{Method as ReqwlessMethod, RequestBuilder};

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
    let url = write_url(builder, url_buf)?;
    let method = to_reqwless_method(builder.method());
    let request_headers = RequestHeaders::from_builder(builder)?;
    let mut headers = [("", ""); MAX_REQUEST_HEADERS];
    let header_count = request_headers.write_into(&mut headers)?;

    let mut request = http
        .request(method, url)
        .await
        .map_err(|_| Error::ConnectionFailed)?;

    if header_count > 0 {
        request = request.headers(&headers[..header_count]);
    }

    let (status, body) = if let Some(body_bytes) = builder.body() {
        let mut request = request
            .body(body_bytes)
            .content_type(ContentType::ApplicationJson);
        let response = request
            .send(rx_buf)
            .await
            .map_err(|_| Error::ConnectionFailed)?;
        let status = response.status.0;
        let body = response
            .body()
            .read_to_end()
            .await
            .map_err(|_| Error::BufferTooSmall)?;
        (status, body)
    } else {
        let response = request
            .send(rx_buf)
            .await
            .map_err(|_| Error::ConnectionFailed)?;
        let status = response.status.0;
        let body = response
            .body()
            .read_to_end()
            .await
            .map_err(|_| Error::BufferTooSmall)?;
        (status, body)
    };

    Ok(SupabaseResponse::new(status, body))
}
