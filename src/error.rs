use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Dns,
    ConnectionFailed,
    Tls,
    Timeout,
    Http(u16),
    Postgrest(PostgrestError),
    BufferTooSmall,
    UrlTooLong,
    JsonParse,
    Utf8,
    InvalidRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostgrestError {
    // PGRST000-003: Connection
    ConnectionError,
    ConnectionPool,

    // PGRST100-128: Request
    InvalidQueryString,
    InvalidBody,
    InvalidRange,
    InvalidUpsert,
    SchemaNotExposed,
    InvalidContentType,
    SingularityViolation,
    MethodNotAllowed,

    // PGRST200-205: Schema cache
    RelationshipError,
    AmbiguousRelation,
    FunctionNotFound,
    TableNotFound,

    // PGRST300-303: Auth
    JwtSecretMissing,
    JwtInvalid,
    AuthRequired,
    JwtClaimsInvalid,

    // Database-level (Postgres error codes)
    ForeignKeyViolation,
    UniqueViolation,
    InsufficientPrivilege,
    UndefinedTable,
    UndefinedFunction,
    ReadOnlyTransaction,

    Unknown(u16),
}

impl PostgrestError {
    pub fn from_pgrst_code(code: &str) -> Self {
        match code {
            "PGRST000" | "PGRST001" | "PGRST002" => Self::ConnectionError,
            "PGRST003" => Self::ConnectionPool,
            "PGRST100" => Self::InvalidQueryString,
            "PGRST102" => Self::InvalidBody,
            "PGRST103" => Self::InvalidRange,
            "PGRST105" | "PGRST114" | "PGRST115" => Self::InvalidUpsert,
            "PGRST106" => Self::SchemaNotExposed,
            "PGRST107" => Self::InvalidContentType,
            "PGRST116" => Self::SingularityViolation,
            "PGRST101" | "PGRST117" => Self::MethodNotAllowed,
            "PGRST200" => Self::RelationshipError,
            "PGRST201" | "PGRST203" => Self::AmbiguousRelation,
            "PGRST202" => Self::FunctionNotFound,
            "PGRST205" | "PGRST125" => Self::TableNotFound,
            "PGRST300" => Self::JwtSecretMissing,
            "PGRST301" => Self::JwtInvalid,
            "PGRST302" => Self::AuthRequired,
            "PGRST303" => Self::JwtClaimsInvalid,
            "23503" => Self::ForeignKeyViolation,
            "23505" => Self::UniqueViolation,
            "42501" => Self::InsufficientPrivilege,
            "42P01" => Self::UndefinedTable,
            "42883" => Self::UndefinedFunction,
            "25006" => Self::ReadOnlyTransaction,
            _ => Self::Unknown(0),
        }
    }

    pub fn from_http_status(status: u16) -> Self {
        match status {
            401 => Self::AuthRequired,
            403 => Self::InsufficientPrivilege,
            404 => Self::TableNotFound,
            405 => Self::MethodNotAllowed,
            409 => Self::UniqueViolation,
            416 => Self::InvalidRange,
            _ => Self::Unknown(status),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dns => f.write_str("dns resolution failed"),
            Self::ConnectionFailed => f.write_str("connection failed"),
            Self::Tls => f.write_str("tls error"),
            Self::Timeout => f.write_str("request timed out"),
            Self::Http(code) => write!(f, "http error: {}", code),
            Self::Postgrest(e) => write!(f, "postgrest: {}", e),
            Self::BufferTooSmall => f.write_str("buffer too small"),
            Self::UrlTooLong => f.write_str("url exceeds buffer capacity"),
            Self::JsonParse => f.write_str("json parse error"),
            Self::Utf8 => f.write_str("invalid utf-8"),
            Self::InvalidRequest => f.write_str("invalid request"),
        }
    }
}

impl fmt::Display for PostgrestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionError => f.write_str("connection error"),
            Self::ConnectionPool => f.write_str("connection pool timeout"),
            Self::InvalidQueryString => f.write_str("invalid query string"),
            Self::InvalidBody => f.write_str("invalid request body"),
            Self::InvalidRange => f.write_str("invalid range"),
            Self::InvalidUpsert => f.write_str("invalid upsert"),
            Self::SchemaNotExposed => f.write_str("schema not exposed"),
            Self::InvalidContentType => f.write_str("invalid content type"),
            Self::SingularityViolation => f.write_str("singularity violation"),
            Self::MethodNotAllowed => f.write_str("method not allowed"),
            Self::RelationshipError => f.write_str("relationship error"),
            Self::AmbiguousRelation => f.write_str("ambiguous relation"),
            Self::FunctionNotFound => f.write_str("function not found"),
            Self::TableNotFound => f.write_str("table not found"),
            Self::JwtSecretMissing => f.write_str("jwt secret missing"),
            Self::JwtInvalid => f.write_str("jwt invalid"),
            Self::AuthRequired => f.write_str("auth required"),
            Self::JwtClaimsInvalid => f.write_str("jwt claims invalid"),
            Self::ForeignKeyViolation => f.write_str("foreign key violation"),
            Self::UniqueViolation => f.write_str("unique violation"),
            Self::InsufficientPrivilege => f.write_str("insufficient privilege"),
            Self::UndefinedTable => f.write_str("undefined table"),
            Self::UndefinedFunction => f.write_str("undefined function"),
            Self::ReadOnlyTransaction => f.write_str("read only transaction"),
            Self::Unknown(code) => write!(f, "unknown error ({})", code),
        }
    }
}
