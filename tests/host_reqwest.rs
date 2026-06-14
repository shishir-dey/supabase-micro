use dotenv::dotenv;
use std::env;
use supabase_micro::postgrest::{Method, PostgrestBuilder};
use supabase_micro::response::SupabaseResponse;
use supabase_micro::transport::request::{write_url, RequestHeaders};
use supabase_micro::SupabaseClient;

fn boxed_library_error(error: supabase_micro::Error) -> Box<dyn std::error::Error> {
    Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        error.to_string(),
    ))
}

struct HostResponse {
    status: u16,
    body: Vec<u8>,
}

impl HostResponse {
    fn as_supabase_response(&self) -> SupabaseResponse<'_> {
        SupabaseResponse::new(self.status, self.body.as_slice())
    }
}

struct SupabaseTestConfig {
    url: String,
    anon_key: String,
    table: String,
    select: String,
}

impl SupabaseTestConfig {
    fn from_env() -> Option<Self> {
        dotenv().ok();

        let url = env::var("SUPABASE_URL").ok();
        let anon_key = env::var("SUPABASE_ANON_KEY").ok();
        let table = env::var("SUPABASE_TEST_TABLE").ok();

        match (url, anon_key, table) {
            (Some(url), Some(anon_key), Some(table)) => Some(Self {
                url,
                anon_key,
                table,
                select: env::var("SUPABASE_TEST_SELECT").unwrap_or_else(|_| "*".to_string()),
            }),
            _ => {
                eprintln!(
                    "skipping host integration test; set SUPABASE_URL, SUPABASE_ANON_KEY, and SUPABASE_TEST_TABLE"
                );
                None
            }
        }
    }
}

async fn execute_reqwest<const N: usize>(
    http: &reqwest::Client,
    builder: &PostgrestBuilder<'_, N>,
) -> Result<HostResponse, Box<dyn std::error::Error>> {
    let mut url_buf = vec![0; builder.base_url().len() + builder.path().len()];
    let url = write_url(builder, &mut url_buf).map_err(boxed_library_error)?;
    let request_headers = RequestHeaders::from_builder(builder).map_err(boxed_library_error)?;

    let method = match builder.method() {
        Method::Get => reqwest::Method::GET,
        Method::Post => reqwest::Method::POST,
        Method::Patch => reqwest::Method::PATCH,
        Method::Delete => reqwest::Method::DELETE,
    };

    let mut request = http.request(method, url);

    for header in request_headers.iter() {
        request = request.header(header.name, header.value);
    }

    if let Some(body) = builder.body() {
        request = request
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body.to_vec());
    }

    let response = request.send().await?;
    let status = response.status().as_u16();
    let body = response.bytes().await?.to_vec();

    Ok(HostResponse { status, body })
}

#[tokio::test]
async fn selects_rows_with_reqwest_host_transport() -> Result<(), Box<dyn std::error::Error>> {
    let Some(config) = SupabaseTestConfig::from_env() else {
        return Ok(());
    };

    let supabase = SupabaseClient::<512>::new(config.url.as_str(), config.anon_key.as_str());
    let mut query = supabase.from(config.table.as_str());
    query
        .select(config.select.as_str())
        .map_err(boxed_library_error)?;
    query.limit(1).map_err(boxed_library_error)?;

    let http = reqwest::Client::new();
    let response = execute_reqwest(&http, &query).await?;
    let supabase_response = response.as_supabase_response();

    assert!(
        supabase_response.is_success(),
        "expected successful Supabase response, got status {} with body {}",
        supabase_response.status,
        String::from_utf8_lossy(supabase_response.body)
    );

    Ok(())
}
