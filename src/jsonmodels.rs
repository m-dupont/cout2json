use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum TypedValue {
    String(String),
    Integer(i64),
    Float(f64),
}

impl Display for TypedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypedValue::String(s) => write!(f, "{}", s),
            TypedValue::Integer(i) => write!(f, "{}", i),
            TypedValue::Float(fl) => write!(f, "{}", fl),
        }
    }
}

/// convert string to TypedValue
impl From<String> for TypedValue {
    fn from(s: String) -> Self {
        if let Ok(i) = i64::from_str_radix(&s, 10) {
            return TypedValue::Integer(i);
        }

        if let Ok(f) = f64::from_str(&s) {
            return TypedValue::Float(f);
        }

        TypedValue::String(s)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum Node {
    Leaf(TypedValue),
    Dict(MapNodes),
    Array(Vec<Node>),
}

fn _get_leafs_from_map(m: &MapNodes) -> HashMap<String, TypedValue> {
    let mut h = HashMap::new();
    for (k, v) in &m.nodes {
        match v {
            Node::Leaf(t) => {
                h.insert(k.clone(), t.clone());
            }
            Node::Dict(m) => {
                let s = k.clone() + ".";
                for (k, v) in _get_leafs_from_map(m) {
                    h.insert(s.clone() + &k, v);
                }
                // let mut h2 = get_leafs_from_map(&m);
                // h.extend(h2);
            }
            Node::Array(_) => {
                todo!("value is an array")
            }
        }
    }
    h
}

#[derive(Debug, PartialEq)]
pub struct MapNodes {
    pub nodes: HashMap<String, Node>,
}

impl MapNodes {
    pub fn new() -> Self {
        MapNodes {
            nodes: HashMap::new(),
        }
    }


    /// For debugging purposes
    pub fn _json_object(&self) -> serde_json::Value {
        serde_json::to_value(&self).unwrap()
    }

    pub fn _leafs(&self) -> HashMap<String, TypedValue> {
        _get_leafs_from_map(&self)
    }
}

impl Serialize for MapNodes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.nodes.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MapNodes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let nodes = HashMap::<String, Node>::deserialize(deserializer)?;
        Ok(MapNodes { nodes })
    }
}

// pub type MapNodes = HashMap<String, Node>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsonmodels::Node::Leaf;
    use serde_json::json;

    #[test]
    fn ser_1_1s() {
        let mut m = MapNodes::new();
        m.nodes
            .insert("a".to_string(), Leaf(TypedValue::String("b".to_string())));

        let correct_json = json!({"a":"b"});
        assert_eq!(m._json_object(), correct_json);
    }

    #[test]
    fn de_1_1s() {
        let json = json!({"a":"b"});
        let m: MapNodes = serde_json::from_value(json).unwrap();
        assert_eq!(m.nodes.len(), 1);
        assert_eq!(
            m.nodes.get("a").unwrap(),
            &Leaf(TypedValue::String("b".to_string()))
        );

        let mut m2 = MapNodes::new();
        m2.nodes
            .insert("a".to_string(), Leaf(TypedValue::String("b".to_string())));

        assert_eq!(m, m2);
    }

    #[test]
    fn de_1_1i() {
        let json = json!({"a":1});
        let m: MapNodes = serde_json::from_value(json).unwrap();
        assert_eq!(m.nodes.len(), 1);
        assert_eq!(m.nodes.get("a").unwrap(), &Leaf(TypedValue::Integer(1)));

        let mut m2 = MapNodes::new();
        m2.nodes
            .insert("a".to_string(), Leaf(TypedValue::Integer(1)));

        assert_eq!(m, m2);
    }

    #[test]
    fn de_1_1f() {
        let json = json!({"a":1.3});
        let m: MapNodes = serde_json::from_value(json).unwrap();
        assert_eq!(m.nodes.len(), 1);
        assert_eq!(m.nodes.get("a").unwrap(), &Leaf(TypedValue::Float(1.3)));

        let mut m2 = MapNodes::new();
        m2.nodes
            .insert("a".to_string(), Leaf(TypedValue::Float(1.3)));

        assert_eq!(m, m2);
    }

    #[test]
    fn get_leaf_1() {
        let json = json!({"a":1});
        let m: MapNodes = serde_json::from_value(json).unwrap();
        assert_eq!(m.nodes.len(), 1);
        let leafs = m._leafs();

        assert_eq!(leafs.len(), 1);
        assert_eq!(leafs.get("a").unwrap(), &TypedValue::Integer(1));
    }

    #[test]
    fn get_leaf_2() {
        let json = json!({"a":1, "b": 2});
        let m: MapNodes = serde_json::from_value(json).unwrap();
        let leafs = m._leafs();

        assert_eq!(leafs.len(), 2);
        assert_eq!(leafs.get("a").unwrap(), &TypedValue::Integer(1));
        assert_eq!(leafs.get("b").unwrap(), &TypedValue::Integer(2));
    }

    #[test]
    fn get_leaf_2_2() {
        let json = json!({"a": {"b": 1, "c": 2}});
        let m: MapNodes = serde_json::from_value(json).unwrap();
        let leafs = m._leafs();

        assert_eq!(leafs.len(), 2);
        assert_eq!(leafs.get("a.b").unwrap(), &TypedValue::Integer(1));
        assert_eq!(leafs.get("a.c").unwrap(), &TypedValue::Integer(2));
    }
}
