/// API key + optional JWT bearer token for Supabase requests.
pub struct Auth<'a> {
    pub api_key: &'a str,
    pub bearer_token: Option<&'a str>,
}

impl<'a> Auth<'a> {
    pub fn new(api_key: &'a str) -> Self {
        Self {
            api_key,
            bearer_token: None,
        }
    }

    pub fn with_bearer(api_key: &'a str, token: &'a str) -> Self {
        Self {
            api_key,
            bearer_token: Some(token),
        }
    }

    pub fn authorization_value<const N: usize>(&self) -> Option<heapless::String<N>> {
        self.bearer_token.and_then(|token| {
            let mut val = heapless::String::new();
            if val.push_str("Bearer ").is_err() {
                return None;
            }
            if val.push_str(token).is_err() {
                return None;
            }
            Some(val)
        })
    }
}
