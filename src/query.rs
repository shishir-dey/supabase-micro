use crate::error::Error;
use core::fmt::Write;
use heapless::String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ordering {
    Asc,
    Desc,
}

/// Builds PostgREST query strings into a fixed-capacity heapless::String<N>.
///
/// # Example
/// ```
/// use supabase_micro::query::{QueryBuilder, Ordering};
///
/// let mut qb = QueryBuilder::<256>::new("/rest/v1/users");
/// qb.select("id,name").unwrap();
/// qb.eq("id", "42").unwrap();
/// qb.limit(10).unwrap();
/// assert_eq!(qb.as_str(), "/rest/v1/users?select=id,name&id=eq.42&limit=10");
/// ```
pub struct QueryBuilder<const N: usize> {
    buf: String<N>,
    has_params: bool,
}

impl<const N: usize> QueryBuilder<N> {
    pub fn new(base_path: &str) -> Self {
        let mut buf = String::new();
        let _ = buf.push_str(base_path);
        Self {
            buf,
            has_params: false,
        }
    }

    pub fn as_str(&self) -> &str {
        self.buf.as_str()
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    fn append_separator(&mut self) -> Result<(), Error> {
        if self.has_params {
            self.buf.push('&').map_err(|_| Error::UrlTooLong)?;
        } else {
            self.buf.push('?').map_err(|_| Error::UrlTooLong)?;
            self.has_params = true;
        }
        Ok(())
    }

    fn append_filter(&mut self, column: &str, op: &str, value: &str) -> Result<&mut Self, Error> {
        self.append_separator()?;
        self.buf.push_str(column).map_err(|_| Error::UrlTooLong)?;
        self.buf.push('=').map_err(|_| Error::UrlTooLong)?;
        self.buf.push_str(op).map_err(|_| Error::UrlTooLong)?;
        self.buf.push('.').map_err(|_| Error::UrlTooLong)?;
        self.buf.push_str(value).map_err(|_| Error::UrlTooLong)?;
        Ok(self)
    }

    fn append_kv(&mut self, key: &str, value: &str) -> Result<&mut Self, Error> {
        self.append_separator()?;
        self.buf.push_str(key).map_err(|_| Error::UrlTooLong)?;
        self.buf.push('=').map_err(|_| Error::UrlTooLong)?;
        self.buf.push_str(value).map_err(|_| Error::UrlTooLong)?;
        Ok(self)
    }

    pub fn select(&mut self, columns: &str) -> Result<&mut Self, Error> {
        self.append_kv("select", columns)
    }

    pub fn eq(&mut self, column: &str, value: &str) -> Result<&mut Self, Error> {
        self.append_filter(column, "eq", value)
    }

    pub fn neq(&mut self, column: &str, value: &str) -> Result<&mut Self, Error> {
        self.append_filter(column, "neq", value)
    }

    pub fn gt(&mut self, column: &str, value: &str) -> Result<&mut Self, Error> {
        self.append_filter(column, "gt", value)
    }

    pub fn gte(&mut self, column: &str, value: &str) -> Result<&mut Self, Error> {
        self.append_filter(column, "gte", value)
    }

    pub fn lt(&mut self, column: &str, value: &str) -> Result<&mut Self, Error> {
        self.append_filter(column, "lt", value)
    }

    pub fn lte(&mut self, column: &str, value: &str) -> Result<&mut Self, Error> {
        self.append_filter(column, "lte", value)
    }

    pub fn like(&mut self, column: &str, pattern: &str) -> Result<&mut Self, Error> {
        self.append_filter(column, "like", pattern)
    }

    pub fn ilike(&mut self, column: &str, pattern: &str) -> Result<&mut Self, Error> {
        self.append_filter(column, "ilike", pattern)
    }

    /// Filter using `in` operator. Value should include parens: `"(1,2,3)"`
    pub fn in_(&mut self, column: &str, values: &str) -> Result<&mut Self, Error> {
        self.append_filter(column, "in", values)
    }

    /// Filter using `is` operator. Value is typically `"null"`, `"true"`, or `"false"`
    pub fn is(&mut self, column: &str, value: &str) -> Result<&mut Self, Error> {
        self.append_filter(column, "is", value)
    }

    pub fn order(&mut self, column: &str, direction: Ordering) -> Result<&mut Self, Error> {
        let dir = match direction {
            Ordering::Asc => "asc",
            Ordering::Desc => "desc",
        };
        self.append_separator()?;
        self.buf.push_str("order=").map_err(|_| Error::UrlTooLong)?;
        self.buf.push_str(column).map_err(|_| Error::UrlTooLong)?;
        self.buf.push('.').map_err(|_| Error::UrlTooLong)?;
        self.buf.push_str(dir).map_err(|_| Error::UrlTooLong)?;
        Ok(self)
    }

    pub fn limit(&mut self, count: usize) -> Result<&mut Self, Error> {
        self.append_separator()?;
        self.buf.push_str("limit=").map_err(|_| Error::UrlTooLong)?;
        write!(self.buf, "{}", count).map_err(|_| Error::UrlTooLong)?;
        Ok(self)
    }

    pub fn offset(&mut self, start: usize) -> Result<&mut Self, Error> {
        self.append_separator()?;
        self.buf
            .push_str("offset=")
            .map_err(|_| Error::UrlTooLong)?;
        write!(self.buf, "{}", start).map_err(|_| Error::UrlTooLong)?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_all() {
        let mut qb = QueryBuilder::<128>::new("/rest/v1/users");
        qb.select("*").unwrap();
        assert_eq!(qb.as_str(), "/rest/v1/users?select=*");
    }

    #[test]
    fn select_columns() {
        let mut qb = QueryBuilder::<128>::new("/rest/v1/users");
        qb.select("id,name,email").unwrap();
        assert_eq!(qb.as_str(), "/rest/v1/users?select=id,name,email");
    }

    #[test]
    fn eq_filter() {
        let mut qb = QueryBuilder::<128>::new("/rest/v1/users");
        qb.select("*").unwrap();
        qb.eq("id", "42").unwrap();
        assert_eq!(qb.as_str(), "/rest/v1/users?select=*&id=eq.42");
    }

    #[test]
    fn multiple_filters() {
        let mut qb = QueryBuilder::<256>::new("/rest/v1/users");
        qb.select("*").unwrap();
        qb.eq("status", "active").unwrap();
        qb.gte("age", "18").unwrap();
        qb.lt("age", "65").unwrap();
        assert_eq!(
            qb.as_str(),
            "/rest/v1/users?select=*&status=eq.active&age=gte.18&age=lt.65"
        );
    }

    #[test]
    fn like_filter() {
        let mut qb = QueryBuilder::<128>::new("/rest/v1/users");
        qb.select("*").unwrap();
        qb.like("name", "*foo*").unwrap();
        assert_eq!(qb.as_str(), "/rest/v1/users?select=*&name=like.*foo*");
    }

    #[test]
    fn ilike_filter() {
        let mut qb = QueryBuilder::<128>::new("/rest/v1/users");
        qb.select("*").unwrap();
        qb.ilike("email", "*@example.com").unwrap();
        assert_eq!(
            qb.as_str(),
            "/rest/v1/users?select=*&email=ilike.*@example.com"
        );
    }

    #[test]
    fn in_filter() {
        let mut qb = QueryBuilder::<128>::new("/rest/v1/users");
        qb.select("*").unwrap();
        qb.in_("id", "(1,2,3)").unwrap();
        assert_eq!(qb.as_str(), "/rest/v1/users?select=*&id=in.(1,2,3)");
    }

    #[test]
    fn is_null_filter() {
        let mut qb = QueryBuilder::<128>::new("/rest/v1/users");
        qb.select("*").unwrap();
        qb.is("deleted_at", "null").unwrap();
        assert_eq!(qb.as_str(), "/rest/v1/users?select=*&deleted_at=is.null");
    }

    #[test]
    fn neq_filter() {
        let mut qb = QueryBuilder::<128>::new("/rest/v1/users");
        qb.neq("role", "admin").unwrap();
        assert_eq!(qb.as_str(), "/rest/v1/users?role=neq.admin");
    }

    #[test]
    fn ordering() {
        let mut qb = QueryBuilder::<128>::new("/rest/v1/users");
        qb.select("*").unwrap();
        qb.order("created_at", Ordering::Desc).unwrap();
        assert_eq!(qb.as_str(), "/rest/v1/users?select=*&order=created_at.desc");
    }

    #[test]
    fn limit_and_offset() {
        let mut qb = QueryBuilder::<128>::new("/rest/v1/users");
        qb.select("*").unwrap();
        qb.limit(10).unwrap();
        qb.offset(20).unwrap();
        assert_eq!(qb.as_str(), "/rest/v1/users?select=*&limit=10&offset=20");
    }

    #[test]
    fn complex_query() {
        let mut qb = QueryBuilder::<512>::new("/rest/v1/users");
        qb.select("id,name,email").unwrap();
        qb.eq("status", "active").unwrap();
        qb.gte("age", "18").unwrap();
        qb.like("name", "J*").unwrap();
        qb.order("name", Ordering::Asc).unwrap();
        qb.limit(25).unwrap();
        qb.offset(0).unwrap();
        assert_eq!(
            qb.as_str(),
            "/rest/v1/users?select=id,name,email&status=eq.active&age=gte.18&name=like.J*&order=name.asc&limit=25&offset=0"
        );
    }

    #[test]
    fn buffer_overflow() {
        let mut qb = QueryBuilder::<32>::new("/rest/v1/users");
        let result = qb.select("id,name,email,avatar,bio,created_at,updated_at");
        assert_eq!(result.unwrap_err(), Error::UrlTooLong);
    }

    #[test]
    fn gt_lte_filters() {
        let mut qb = QueryBuilder::<128>::new("/rest/v1/items");
        qb.gt("price", "100").unwrap();
        qb.lte("price", "500").unwrap();
        assert_eq!(qb.as_str(), "/rest/v1/items?price=gt.100&price=lte.500");
    }
}
