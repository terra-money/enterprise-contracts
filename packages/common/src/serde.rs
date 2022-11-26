use std::any::type_name;

use cosmwasm_std::{from_slice, Binary, StdError, StdResult};
use serde::{de::DeserializeOwned, Serialize};

pub trait SerdeExt {
    fn to_vec(&self) -> StdResult<Vec<u8>>
    where
        Self: Sized + Serialize,
    {
        serde_json_wasm::to_vec(self).map_err(|e| StdError::serialize_err(type_name::<Self>(), e))
    }

    fn to_binary(&self) -> StdResult<Binary>
    where
        Self: Sized + Serialize,
    {
        self.to_vec().map(Binary)
    }
}

pub trait DeserdeExt {
    fn to_t<T: DeserializeOwned>(&self) -> StdResult<T>;
}

impl DeserdeExt for Binary {
    fn to_t<T: DeserializeOwned>(&self) -> StdResult<T> {
        from_slice(self.as_slice())
    }
}

#[cfg(test)]
pub mod tests {
    use serde::{Deserialize, Serialize};

    use common_derive::SerdeExt;

    use super::*;

    #[derive(Serialize, Deserialize, SerdeExt, Debug, PartialEq)]
    #[serde(rename_all = "snake_case")]
    enum SomeMsg {
        Refund {},
        ReleaseAll {
            image: String,
            amount: u32,
            time: u64,
            karma: i32,
        },
        Cowsay {
            text: String,
        },
    }

    #[test]
    fn to_vec_works() {
        let msg = SomeMsg::Refund {};
        let serialized = msg.to_vec().unwrap();
        assert_eq!(serialized, br#"{"refund":{}}"#);

        let msg = SomeMsg::ReleaseAll {
            image: "foo".to_string(),
            amount: 42,
            time: 9007199254740999, // Number.MAX_SAFE_INTEGER + 7
            karma: -17,
        };
        let serialized = String::from_utf8(msg.to_vec().unwrap()).unwrap();
        assert_eq!(
            serialized,
            r#"{"release_all":{"image":"foo","amount":42,"time":9007199254740999,"karma":-17}}"#
        );
    }

    #[test]
    fn to_t_works() {
        let msg = SomeMsg::Refund {};
        let serialized = msg.to_binary().unwrap();

        let parse_binary = serialized.to_t::<SomeMsg>().unwrap();
        assert_eq!(parse_binary, msg);
    }
}
