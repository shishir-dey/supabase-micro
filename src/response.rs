use crate::error::{Error, PostgrestError};
use serde::Deserialize;

pub struct SupabaseResponse<'a> {
    pub status: u16,
    pub body: &'a [u8],
}

impl<'a> SupabaseResponse<'a> {
    pub fn new(status: u16, body: &'a [u8]) -> Self {
        Self { status, body }
    }

    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    pub fn deserialize<T: Deserialize<'a>>(&self) -> Result<(T, usize), Error> {
        if !self.is_success() {
            return Err(self.to_error());
        }
        serde_json_core::from_slice(self.body).map_err(|_| Error::JsonParse)
    }

    pub fn to_error(&self) -> Error {
        if self.is_success() {
            return Error::InvalidRequest;
        }
        if let Ok((err_body, _)) = serde_json_core::from_slice::<ErrorBody<'_>>(self.body) {
            let pg_err = PostgrestError::from_pgrst_code(err_body.code);
            if pg_err != PostgrestError::Unknown(0) {
                return Error::Postgrest(pg_err);
            }
        }
        Error::Postgrest(PostgrestError::from_http_status(self.status))
    }
}

#[derive(Deserialize)]
struct ErrorBody<'a> {
    code: &'a str,
    #[serde(default)]
    #[allow(dead_code)]
    message: Option<&'a str>,
}

pub fn map_status(status: u16) -> Result<(), Error> {
    if (200..300).contains(&status) {
        Ok(())
    } else {
        Err(Error::Postgrest(PostgrestError::from_http_status(status)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_status() {
        assert!(map_status(200).is_ok());
        assert!(map_status(201).is_ok());
        assert!(map_status(204).is_ok());
    }

    #[test]
    fn error_status() {
        let err = map_status(401).unwrap_err();
        assert_eq!(err, Error::Postgrest(PostgrestError::AuthRequired));

        let err = map_status(409).unwrap_err();
        assert_eq!(err, Error::Postgrest(PostgrestError::UniqueViolation));

        let err = map_status(404).unwrap_err();
        assert_eq!(err, Error::Postgrest(PostgrestError::TableNotFound));
    }

    #[test]
    fn parse_postgrest_error_body() {
        let body = br#"{"code":"PGRST301","message":"JWT expired"}"#;
        let resp = SupabaseResponse::new(401, body);
        let err = resp.to_error();
        assert_eq!(err, Error::Postgrest(PostgrestError::JwtInvalid));
    }

    #[test]
    fn parse_postgres_error_code() {
        let body = br#"{"code":"23505","message":"duplicate key"}"#;
        let resp = SupabaseResponse::new(409, body);
        let err = resp.to_error();
        assert_eq!(err, Error::Postgrest(PostgrestError::UniqueViolation));
    }

    #[test]
    fn parse_foreign_key_violation() {
        let body = br#"{"code":"23503","message":"violates foreign key constraint"}"#;
        let resp = SupabaseResponse::new(409, body);
        let err = resp.to_error();
        assert_eq!(err, Error::Postgrest(PostgrestError::ForeignKeyViolation));
    }

    #[test]
    fn fallback_to_http_status() {
        let body = b"not json";
        let resp = SupabaseResponse::new(403, body);
        let err = resp.to_error();
        assert_eq!(err, Error::Postgrest(PostgrestError::InsufficientPrivilege));
    }

    #[test]
    fn deserialize_success() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct User<'a> {
            id: u32,
            name: &'a str,
        }

        let body = br#"{"id":42,"name":"Alice"}"#;
        let resp = SupabaseResponse::new(200, body);
        let (user, _) = resp.deserialize::<User<'_>>().unwrap();
        assert_eq!(user.id, 42);
        assert_eq!(user.name, "Alice");
    }

    #[test]
    fn deserialize_error_status() {
        #[derive(Deserialize)]
        struct User {
            #[allow(dead_code)]
            id: u32,
        }

        let body = br#"{"code":"PGRST302","message":"auth required"}"#;
        let resp = SupabaseResponse::new(401, body);
        let result = resp.deserialize::<User>();
        assert!(result.is_err());
    }
}
