//! Small dynamic YAML model, deserialized by a maintained safe parser.
//! Retains numeric mapping keys used by Cardigann, unlike JSON-only models.
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Sequence(Vec<Value>),
    Mapping(Mapping),
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mapping(Vec<(Value, Value)>);
impl Mapping {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert(&mut self, key: Value, value: Value) {
        if let Some(entry) = self.0.iter_mut().find(|entry| entry.0 == key) {
            entry.1 = value;
        } else {
            self.0.push((key, value));
        }
    }
    pub fn get(&self, key: Value) -> Option<&Value> {
        self.0
            .iter()
            .find(|entry| entry.0 == key)
            .map(|entry| &entry.1)
    }
    pub fn iter(&self) -> std::slice::Iter<'_, (Value, Value)> {
        self.0.iter()
    }
}
impl<'a> IntoIterator for &'a Mapping {
    type Item = &'a (Value, Value);
    type IntoIter = std::slice::Iter<'a, (Value, Value)>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
impl Value {
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(value) = self {
            Some(*value)
        } else {
            None
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_sequence(&self) -> Option<&Vec<Value>> {
        if let Self::Sequence(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn as_mapping(&self) -> Option<&Mapping> {
        if let Self::Mapping(v) = self {
            Some(v)
        } else {
            None
        }
    }
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
    pub fn is_mapping(&self) -> bool {
        matches!(self, Self::Mapping(_))
    }
}
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Dynamic;
        impl<'de> Visitor<'de> for Dynamic {
            type Value = Value;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a YAML value")
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }
            fn visit_none<E: serde::de::Error>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }
            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Value, E> {
                Ok(Value::Bool(v))
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Value, E> {
                Ok(Value::Number(v.into()))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Value, E> {
                Ok(Value::Number(v.into()))
            }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Value, E> {
                serde_json::Number::from_f64(v)
                    .map(Value::Number)
                    .ok_or_else(|| E::custom("Non-finite number"))
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Value, E> {
                Ok(Value::String(v.into()))
            }
            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Value, E> {
                Ok(Value::String(v))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Value, A::Error> {
                let mut values = Vec::new();
                while let Some(v) = a.next_element()? {
                    values.push(v)
                }
                Ok(Value::Sequence(values))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Value, A::Error> {
                let mut map = Mapping::new();
                while let Some((k, v)) = a.next_entry()? {
                    map.insert(k, v)
                }
                Ok(Value::Mapping(map))
            }
        }
        deserializer.deserialize_any(Dynamic)
    }
}
pub fn from_str<T: serde::de::DeserializeOwned>(input: &str) -> Result<T, serde_saphyr::Error> {
    serde_saphyr::from_str(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn numeric_keys_and_aliases_remain_compatible() {
        let value: Value =
            from_str("defaults: &defaults [one, two]\ncaps:\n  2000: Movies\npaths: *defaults\n")
                .unwrap();
        let root = value.as_mapping().unwrap();
        assert_eq!(
            root.get(Value::String("paths".into()))
                .unwrap()
                .as_sequence()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            root.get(Value::String("caps".into()))
                .unwrap()
                .as_mapping()
                .unwrap()
                .get(Value::Number(2000.into()))
                .unwrap()
                .as_str(),
            Some("Movies")
        );
    }
    #[test]
    fn bundled_catalogue_parses_without_the_abandoned_parser() {
        for entry in std::fs::read_dir(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/config/indexers/upstream"
        ))
        .unwrap()
        .flatten()
        {
            if entry.path().extension().is_some_and(|ext| ext == "yml") {
                let content = std::fs::read_to_string(entry.path()).unwrap();
                assert!(
                    from_str::<Value>(&content).is_ok(),
                    "{}",
                    entry.path().display()
                );
            }
        }
    }
}
