use std::ops::Deref;

use cosmwasm_schema::cw_serde;

/// A token ID according to the ICS-721 spec. The newtype pattern is
/// used here to provide some distinction between token and class IDs
/// in the type system.
#[cw_serde]
pub struct TokenId(pub String);

/// A class ID according to the ICS-721 spec. The newtype pattern is
/// used here to provide some distinction between token and class IDs
/// in the type system.
#[cw_serde]
pub struct ClassId(pub String);

impl TokenId {
    pub fn new<T>(token_id: T) -> Self
    where
        T: Into<String>,
    {
        Self(token_id.into())
    }
}

impl ClassId {
    pub fn new<T>(class_id: T) -> Self
    where
        T: Into<String>,
    {
        Self(class_id.into())
    }
}

// Allow ClassId to be inferred into String
impl From<ClassId> for String {
    fn from(c: ClassId) -> Self {
        c.0
    }
}

impl From<TokenId> for String {
    fn from(t: TokenId) -> Self {
        t.0
    }
}

impl Deref for ClassId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for ClassId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
