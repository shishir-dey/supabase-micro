use crate::error::Error;
use crate::postgrest::PostgrestBuilder;

pub const MAX_REQUEST_HEADERS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestHeader<'a> {
    pub name: &'static str,
    pub value: &'a str,
}

pub struct RequestHeaders<'a> {
    api_key: Option<&'a str>,
    bearer: heapless::String<512>,
    has_bearer: bool,
    prefer: heapless::String<128>,
    has_prefer: bool,
}

impl<'a> RequestHeaders<'a> {
    pub fn from_builder<const N: usize>(builder: &PostgrestBuilder<'a, N>) -> Result<Self, Error> {
        let mut headers = Self {
            api_key: None,
            bearer: heapless::String::new(),
            has_bearer: false,
            prefer: heapless::String::new(),
            has_prefer: false,
        };

        if let Some(auth) = builder.auth_ref() {
            headers.api_key = Some(auth.api_key);

            if let Some(token) = auth.bearer_token {
                headers
                    .bearer
                    .push_str("Bearer ")
                    .map_err(|_| Error::BufferTooSmall)?;
                headers
                    .bearer
                    .push_str(token)
                    .map_err(|_| Error::BufferTooSmall)?;
                headers.has_bearer = true;
            }
        }

        if builder.wants_upsert() {
            headers
                .prefer
                .push_str("resolution=merge-duplicates")
                .map_err(|_| Error::BufferTooSmall)?;
            headers.has_prefer = true;
        }

        if builder.wants_return() {
            headers.push_prefer_separator()?;
            headers
                .prefer
                .push_str("return=representation")
                .map_err(|_| Error::BufferTooSmall)?;
            headers.has_prefer = true;
        }

        if builder.wants_count() {
            headers.push_prefer_separator()?;
            headers
                .prefer
                .push_str("count=exact")
                .map_err(|_| Error::BufferTooSmall)?;
            headers.has_prefer = true;
        }

        Ok(headers)
    }

    pub fn iter(&self) -> RequestHeadersIter<'_> {
        RequestHeadersIter {
            headers: self,
            index: 0,
        }
    }

    pub fn write_into<'b>(&'b self, out: &mut [(&'static str, &'b str)]) -> Result<usize, Error> {
        let mut count = 0;

        for header in self.iter() {
            if count >= out.len() {
                return Err(Error::BufferTooSmall);
            }

            out[count] = (header.name, header.value);
            count += 1;
        }

        Ok(count)
    }

    fn push_prefer_separator(&mut self) -> Result<(), Error> {
        if self.has_prefer {
            self.prefer
                .push_str(", ")
                .map_err(|_| Error::BufferTooSmall)?;
        }

        Ok(())
    }
}

pub struct RequestHeadersIter<'a> {
    headers: &'a RequestHeaders<'a>,
    index: u8,
}

impl<'a> Iterator for RequestHeadersIter<'a> {
    type Item = RequestHeader<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let header = match self.index {
                0 => self.headers.api_key.map(|value| RequestHeader {
                    name: "apikey",
                    value,
                }),
                1 => {
                    if self.headers.has_bearer {
                        Some(RequestHeader {
                            name: "Authorization",
                            value: self.headers.bearer.as_str(),
                        })
                    } else {
                        None
                    }
                }
                2 => {
                    if self.headers.has_prefer {
                        Some(RequestHeader {
                            name: "Prefer",
                            value: self.headers.prefer.as_str(),
                        })
                    } else {
                        None
                    }
                }
                _ => return None,
            };

            self.index += 1;

            if header.is_some() {
                return header;
            }
        }
    }
}

pub fn write_url<'a, const N: usize>(
    builder: &PostgrestBuilder<'_, N>,
    url_buf: &'a mut [u8],
) -> Result<&'a str, Error> {
    let base = builder.base_url();
    let path = builder.path();
    let total_len = base.len() + path.len();

    if total_len > url_buf.len() {
        return Err(Error::UrlTooLong);
    }

    url_buf[..base.len()].copy_from_slice(base.as_bytes());
    url_buf[base.len()..total_len].copy_from_slice(path.as_bytes());
    core::str::from_utf8(&url_buf[..total_len]).map_err(|_| Error::Utf8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::SupabaseClient;

    #[test]
    fn writes_full_url() {
        let client = SupabaseClient::<128>::new("https://x.supabase.co", "key");
        let mut query = client.from("users");
        query.select("*").unwrap();

        let mut buf = [0; 128];
        let url = write_url(&query, &mut buf).unwrap();

        assert_eq!(url, "https://x.supabase.co/rest/v1/users?select=*");
    }

    #[test]
    fn writes_headers_without_binding_to_transport() {
        let client = SupabaseClient::<128>::with_auth("https://x.supabase.co", "key", "jwt");
        let mut query = client.from("users");
        query.insert(br#"{"name":"Ada"}"#);
        query.count();

        let headers = RequestHeaders::from_builder(&query).unwrap();
        let mut out = [("", ""); MAX_REQUEST_HEADERS];
        let count = headers.write_into(&mut out).unwrap();

        assert_eq!(count, 3);
        assert_eq!(out[0], ("apikey", "key"));
        assert_eq!(out[1], ("Authorization", "Bearer jwt"));
        assert_eq!(out[2], ("Prefer", "return=representation, count=exact"));
    }

    #[test]
    fn upsert_prefer_preserves_postgrest_order() {
        let client = SupabaseClient::<128>::new("https://x.supabase.co", "key");
        let mut query = client.from("users");
        query.upsert(br#"{"id":1}"#);

        let headers = RequestHeaders::from_builder(&query).unwrap();
        let prefer = headers
            .iter()
            .find(|header| header.name == "Prefer")
            .map(|header| header.value);

        assert_eq!(
            prefer,
            Some("resolution=merge-duplicates, return=representation")
        );
    }
}
