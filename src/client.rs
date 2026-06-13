use crate::auth::Auth;
use crate::postgrest::PostgrestBuilder;

/// Top-level Supabase client. Holds configuration (URL + auth) without
/// owning the HTTP transport — keeping it free of platform-specific generics.
///
/// # Type Parameter
/// * `N` — max URL buffer capacity in bytes (default 256)
///
/// # Example
/// ```rust,no_run
/// use supabase_micro::client::SupabaseClient;
///
/// let client = SupabaseClient::<256>::new(
///     "https://example.supabase.co",
///     "my-anon-key",
/// );
/// let mut builder = client.from("users");
/// builder.select("*").unwrap();
/// builder.eq("id", "42").unwrap();
/// // Then pass builder + HttpClient to transport::reqwless_transport::execute()
/// ```
pub struct SupabaseClient<'a, const N: usize = 256> {
    base_url: &'a str,
    auth: Auth<'a>,
}

impl<'a, const N: usize> SupabaseClient<'a, N> {
    pub fn new(url: &'a str, api_key: &'a str) -> Self {
        Self {
            base_url: url,
            auth: Auth::new(api_key),
        }
    }

    pub fn with_auth(url: &'a str, api_key: &'a str, jwt: &'a str) -> Self {
        Self {
            base_url: url,
            auth: Auth::with_bearer(api_key, jwt),
        }
    }

    pub fn from(&'a self, table: &'a str) -> PostgrestBuilder<'a, N> {
        let mut builder = PostgrestBuilder::new(self.base_url, table);
        builder.set_auth(&self.auth);
        builder
    }

    pub fn base_url(&self) -> &str {
        self.base_url
    }

    pub fn auth(&self) -> &Auth<'a> {
        &self.auth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_client() {
        let client = SupabaseClient::<256>::new("https://x.supabase.co", "test-key");
        assert_eq!(client.base_url(), "https://x.supabase.co");
        assert_eq!(client.auth().api_key, "test-key");
        assert!(client.auth().bearer_token.is_none());
    }

    #[test]
    fn create_client_with_jwt() {
        let client =
            SupabaseClient::<256>::with_auth("https://x.supabase.co", "test-key", "jwt-token-here");
        assert_eq!(client.auth().bearer_token, Some("jwt-token-here"));
    }

    #[test]
    fn from_returns_builder() {
        let client = SupabaseClient::<256>::new("https://x.supabase.co", "test-key");
        let mut builder = client.from("users");
        builder.select("*").unwrap();
        assert_eq!(builder.path(), "/rest/v1/users?select=*");
        assert_eq!(builder.base_url(), "https://x.supabase.co");
    }

    #[test]
    fn from_with_filters() {
        let client = SupabaseClient::<256>::new("https://x.supabase.co", "test-key");
        let mut builder = client.from("users");
        builder.select("id,name").unwrap();
        builder.eq("status", "active").unwrap();
        builder.limit(10).unwrap();
        assert_eq!(
            builder.path(),
            "/rest/v1/users?select=id,name&status=eq.active&limit=10"
        );
    }
}
