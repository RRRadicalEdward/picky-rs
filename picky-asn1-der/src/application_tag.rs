use crate::misc::Length;
use crate::Asn1RawDer;
use picky_asn1::tag::{Tag, TagPeeker};
use serde::de::{Error, SeqAccess};
use serde::{de, ser};
use std::fmt;
use std::fmt::Debug;

#[derive(Debug, PartialEq)]
pub struct ApplicationTag<V: Debug + PartialEq, const T: u8>(V);

impl<V: Debug + PartialEq, const T: u8> ApplicationTag<V, T> {
    pub fn from(value: V) -> Self {
        Self(value)
    }
}

impl<'de, V: de::Deserialize<'de> + Debug + PartialEq, const T: u8> de::Deserialize<'de> for ApplicationTag<V, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor<E>(Option<E>, u8);

        impl<E> Visitor<E> {
            pub fn new(tag: u8) -> Self {
                Self(None, tag)
            }
        }

        impl<'de, E: de::Deserialize<'de> + Debug + PartialEq> de::Visitor<'de> for Visitor<E> {
            type Value = E;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(&format!("A valid DER-encoded ApplicationTag{}", self.1))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let tag_peeker: TagPeeker = seq
                    .next_element()
                    .map_err(|e| A::Error::custom(format!("Cannot deserialize application tag: {:?}", e)))?
                    .ok_or_else(|| A::Error::missing_field("ApplicationTag"))?;
                let tag = tag_peeker.next_tag;

                if !tag.is_application() {
                    return Err(A::Error::custom(format!(
                        "Expected Application class tag but got: {:?}",
                        tag.class()
                    )));
                }

                if tag.number() != self.1 {
                    return Err(A::Error::custom(format!(
                        "Expected Application number tag {} but got: {}",
                        self.1,
                        tag.number()
                    )));
                }

                #[derive(Debug, serde::Deserialize)]
                struct ApplicationTagInner<V: Debug> {
                    value: V,
                }

                let rest: ApplicationTagInner<E> = seq
                    .next_element()
                    .map_err(|e| A::Error::custom(format!("Cannot deserialize application tag inner value: {:?}", e)))?
                    .ok_or_else(|| A::Error::missing_field("ApplicationInnerValue"))?;

                Ok(rest.value)
            }
        }

        let inner = deserializer
            .deserialize_enum("ApplicationTag", &["ApplicationTag"], Visitor::<V>::new(T))
            .map_err(D::Error::custom)?;

        Ok(Self(inner))
    }
}

impl<V: ser::Serialize + Debug + PartialEq, const T: u8> ser::Serialize for ApplicationTag<V, T> {
    fn serialize<S>(&self, serializer: S) -> Result<<S as ser::Serializer>::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        use serde::ser::Error;

        let mut buff = Vec::new();
        {
            let mut s = crate::Serializer::new_to_byte_buf(&mut buff);
            self.0
                .serialize(&mut s)
                .map_err(|e| S::Error::custom(format!("Cannot serialize Application tag inner value: {:?}", e)))?;
        }

        let mut res = vec![Tag::application_constructed(T).inner()];

        Length::serialize(buff.len(), &mut res)
            .map_err(|e| S::Error::custom(format!("Cannot serialize Length: {:?}", e)))?;
        res.extend_from_slice(&buff);

        Asn1RawDer(res).serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use crate::application_tag::ApplicationTag;
    use picky_asn1::restricted_string::Utf8String;
    use picky_asn1::wrapper::Utf8StringAsn1;

    #[test]
    fn test_application_tag() {
        let expected_raw = vec![106, 13, 12, 11, 101, 120, 97, 109, 112, 108, 101, 46, 99, 111, 109];
        let expected: ApplicationTag<Utf8StringAsn1, 10> = ApplicationTag::from(Utf8StringAsn1::from(
            Utf8String::from_string("example.com".to_owned()).unwrap(),
        ));

        let app_10: ApplicationTag<Utf8StringAsn1, 10> = crate::from_bytes(&expected_raw).unwrap();
        let app_10_raw = crate::to_vec(&app_10).unwrap();

        assert_eq!(expected, app_10);
        assert_eq!(expected_raw, app_10_raw);
    }
}
