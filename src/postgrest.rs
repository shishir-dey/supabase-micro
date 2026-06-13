use crate::auth::Auth;
use crate::error::Error;
use crate::query::{Ordering, QueryBuilder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Patch,
    Delete,
}

/// Builds a PostgREST request with method, path, query filters, and optional body.
///
/// # Example
/// ```
/// use supabase_micro::postgrest::PostgrestBuilder;
///
/// let mut builder = PostgrestBuilder::<256>::new("https://example.supabase.co", "users");
/// builder.select("*").unwrap();
/// builder.eq("id", "1").unwrap();
/// assert!(builder.path().contains("select=*"));
/// assert!(builder.path().contains("id=eq.1"));
/// ```
pub struct PostgrestBuilder<'a, const N: usize> {
    base_url: &'a str,
    auth: Option<&'a Auth<'a>>,
    query: QueryBuilder<N>,
    method: Method,
    body: Option<&'a [u8]>,
    prefer_return: bool,
    prefer_count: bool,
    prefer_upsert: bool,
}

impl<'a, const N: usize> PostgrestBuilder<'a, N> {
    pub fn new(base_url: &'a str, table: &'a str) -> Self {
        let mut path_buf = heapless::String::<N>::new();
        let _ = path_buf.push_str("/rest/v1/");
        let _ = path_buf.push_str(table);

        Self {
            base_url,
            auth: None,
            query: QueryBuilder::new(path_buf.as_str()),
            method: Method::Get,
            body: None,
            prefer_return: false,
            prefer_count: false,
            prefer_upsert: false,
        }
    }

    pub fn set_auth(&mut self, auth: &'a Auth<'a>) -> &mut Self {
        self.auth = Some(auth);
        self
    }

    pub fn method(&self) -> Method {
        self.method
    }

    pub fn path(&self) -> &str {
        self.query.as_str()
    }

    pub fn base_url(&self) -> &'a str {
        self.base_url
    }

    pub fn body(&self) -> Option<&'a [u8]> {
        self.body
    }

    pub fn auth_ref(&self) -> Option<&'a Auth<'a>> {
        self.auth
    }

    pub fn wants_return(&self) -> bool {
        self.prefer_return
    }

    pub fn wants_count(&self) -> bool {
        self.prefer_count
    }

    pub fn wants_upsert(&self) -> bool {
        self.prefer_upsert
    }

    // --- Operation methods ---

    pub fn select(&mut self, columns: &str) -> Result<&mut Self, Error> {
        self.method = Method::Get;
        self.query.select(columns)?;
        Ok(self)
    }

    pub fn insert(&mut self, body: &'a [u8]) -> &mut Self {
        self.method = Method::Post;
        self.body = Some(body);
        self.prefer_return = true;
        self
    }

    pub fn update(&mut self, body: &'a [u8]) -> &mut Self {
        self.method = Method::Patch;
        self.body = Some(body);
        self.prefer_return = true;
        self
    }

    pub fn delete(&mut self) -> &mut Self {
        self.method = Method::Delete;
        self
    }

    pub fn upsert(&mut self, body: &'a [u8]) -> &mut Self {
        self.method = Method::Post;
        self.body = Some(body);
        self.prefer_upsert = true;
        self.prefer_return = true;
        self
    }

    pub fn returning(&mut self) -> &mut Self {
        self.prefer_return = true;
        self
    }

    pub fn count(&mut self) -> &mut Self {
        self.prefer_count = true;
        self
    }

    // --- Filter methods delegate to QueryBuilder ---

    pub fn eq(&mut self, column: &str, value: &str) -> Result<&mut Self, Error> {
        self.query.eq(column, value)?;
        Ok(self)
    }

    pub fn neq(&mut self, column: &str, value: &str) -> Result<&mut Self, Error> {
        self.query.neq(column, value)?;
        Ok(self)
    }

    pub fn gt(&mut self, column: &str, value: &str) -> Result<&mut Self, Error> {
        self.query.gt(column, value)?;
        Ok(self)
    }

    pub fn gte(&mut self, column: &str, value: &str) -> Result<&mut Self, Error> {
        self.query.gte(column, value)?;
        Ok(self)
    }

    pub fn lt(&mut self, column: &str, value: &str) -> Result<&mut Self, Error> {
        self.query.lt(column, value)?;
        Ok(self)
    }

    pub fn lte(&mut self, column: &str, value: &str) -> Result<&mut Self, Error> {
        self.query.lte(column, value)?;
        Ok(self)
    }

    pub fn like(&mut self, column: &str, pattern: &str) -> Result<&mut Self, Error> {
        self.query.like(column, pattern)?;
        Ok(self)
    }

    pub fn ilike(&mut self, column: &str, pattern: &str) -> Result<&mut Self, Error> {
        self.query.ilike(column, pattern)?;
        Ok(self)
    }

    pub fn in_(&mut self, column: &str, values: &str) -> Result<&mut Self, Error> {
        self.query.in_(column, values)?;
        Ok(self)
    }

    pub fn is(&mut self, column: &str, value: &str) -> Result<&mut Self, Error> {
        self.query.is(column, value)?;
        Ok(self)
    }

    pub fn order(&mut self, column: &str, direction: Ordering) -> Result<&mut Self, Error> {
        self.query.order(column, direction)?;
        Ok(self)
    }

    pub fn limit(&mut self, count: usize) -> Result<&mut Self, Error> {
        self.query.limit(count)?;
        Ok(self)
    }

    pub fn offset(&mut self, start: usize) -> Result<&mut Self, Error> {
        self.query.offset(start)?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_query() {
        let mut b = PostgrestBuilder::<256>::new("https://x.supabase.co", "users");
        b.select("*").unwrap();
        assert_eq!(b.path(), "/rest/v1/users?select=*");
        assert_eq!(b.method(), Method::Get);
    }

    #[test]
    fn select_with_filters() {
        let mut b = PostgrestBuilder::<256>::new("https://x.supabase.co", "users");
        b.select("id,name").unwrap();
        b.eq("id", "42").unwrap();
        b.limit(1).unwrap();
        assert_eq!(b.path(), "/rest/v1/users?select=id,name&id=eq.42&limit=1");
    }

    #[test]
    fn insert_sets_post() {
        let body = br#"{"name":"Bob"}"#;
        let mut b = PostgrestBuilder::<256>::new("https://x.supabase.co", "users");
        b.insert(body);
        assert_eq!(b.method(), Method::Post);
        assert_eq!(b.body(), Some(body.as_slice()));
        assert!(b.wants_return());
    }

    #[test]
    fn update_sets_patch() {
        let body = br#"{"name":"Bob"}"#;
        let mut b = PostgrestBuilder::<256>::new("https://x.supabase.co", "users");
        b.update(body);
        b.eq("id", "1").unwrap();
        assert_eq!(b.method(), Method::Patch);
        assert!(b.path().contains("id=eq.1"));
    }

    #[test]
    fn delete_sets_method() {
        let mut b = PostgrestBuilder::<256>::new("https://x.supabase.co", "users");
        b.delete();
        b.eq("id", "1").unwrap();
        assert_eq!(b.method(), Method::Delete);
    }

    #[test]
    fn upsert_sets_prefer() {
        let body = br#"{"id":1,"name":"Bob"}"#;
        let mut b = PostgrestBuilder::<256>::new("https://x.supabase.co", "users");
        b.upsert(body);
        assert_eq!(b.method(), Method::Post);
        assert!(b.wants_upsert());
        assert!(b.wants_return());
    }
}
