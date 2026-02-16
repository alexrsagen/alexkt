use core::fmt;

use num_bigint::BigInt;
use num_traits::Num;
use serde::{Serialize, Deserialize};
use serde::{ser, de};

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct SerdeBigInt(
    #[serde(serialize_with = "serialize_bigint", deserialize_with = "deserialize_bigint")]
    pub BigInt,
);

impl core::ops::Deref for SerdeBigInt {
    type Target = BigInt;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for SerdeBigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub fn deserialize_bigint<'de, D>(deserializer: D) -> Result<BigInt, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct BigIntStringVisitor;

    impl<'de> de::Visitor<'de> for BigIntStringVisitor {
        type Value = BigInt;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing a big integer")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            BigInt::from_str_radix(v, 10).map_err(E::custom)
        }
    }

    deserializer.deserialize_str(BigIntStringVisitor)
}

pub fn serialize_bigint<S>(value: &BigInt, serializer: S) -> Result<S::Ok, S::Error>
where
	S: ser::Serializer,
{
	serializer.serialize_str(value.to_str_radix(10).as_str())
}