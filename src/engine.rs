use crate::jsonmodels::{MapNodes, Node};
use std::collections::hash_map::Entry;
pub use crate::engine::engine_options::EngineOptions;
use std::process::exit;

pub(crate) mod engine_options;
pub mod errors;

use crate::engine::engine_options::HowToDictInArray;
pub use errors::Error;
pub use errors::Result;

pub fn string_to_dict(mut dotted_keys: Vec<String>, val: Node) -> MapNodes {
    let mut h = MapNodes::new();
    let k = dotted_keys.pop().unwrap();

    h.nodes.insert(k, val);
    if dotted_keys.len() == 0 {
        return h;
    }
    string_to_dict(dotted_keys, Node::Dict(h))
}
pub fn extend_hashmap(h: &mut MapNodes, mut h2: MapNodes, options: &EngineOptions) -> Result<()> {
    for (k, value_to_insert) in h2.nodes.drain() {
        match h.nodes.entry(k.clone()) {
            Entry::Occupied(mut existing_entry) => {
                if options.verbosity > 0 {
                    eprintln!("key {} already exists", existing_entry.key());
                    eprintln!("value = {:?}", existing_entry.get());
                    eprintln!("value to insert = {:?}", value_to_insert);
                }
                match existing_entry.get_mut() {
                    Node::Leaf(s) => {
                        match value_to_insert {
                            Node::Leaf(s_to_insert) => {
                                // convert existing value to array
                                let mut a = Vec::new();
                                a.push(Node::Leaf(s.clone()));
                                a.push(Node::Leaf(s_to_insert));
                                *existing_entry.get_mut() = Node::Array(a);
                            }
                            Node::Dict(d_to_insert) => {
                                let mut h = MapNodes::new();
                                h.nodes.insert("value".to_string(), Node::Leaf(s.clone()));
                                h.nodes.extend(d_to_insert.nodes);
                                *existing_entry.get_mut() = Node::Dict(h);
                            }
                            Node::Array(_) => {
                                todo!("value is an array")
                            }
                        }
                    }
                    Node::Dict(d) => match value_to_insert {
                        Node::Leaf(_) => {
                            todo!("value is a string")
                        }
                        Node::Dict(d2) => {
                            extend_hashmap(d, d2, &options)?;
                        }
                        Node::Array(_) => {
                            todo!("value is an array")
                        }
                    },
                    Node::Array(existing_array) => match value_to_insert {
                        Node::Leaf(s_to_insert) => {
                            existing_array.push(Node::Leaf(s_to_insert));
                        }
                        Node::Dict(d) => match options.how_to_dict_in_array {
                            HowToDictInArray::GenerateError => return Err(Error::HowToDictInArray),
                            HowToDictInArray::MergeDictInArray => {
                                existing_array.push(Node::Dict(d));
                            }
                            HowToDictInArray::MakeArrayAsDictValue => {
                                unimplemented!()
                            }
                        },
                        Node::Array(_) => {
                            todo!("value to insert is an array")
                        }
                    },
                }
            }
            Entry::Vacant(e) => {
                e.insert(value_to_insert);
            }
        }
    }
    Ok(())
}

pub struct Engine {
    options: EngineOptions,
    pub values: MapNodes,
}

impl Engine {
    pub fn new(options: EngineOptions) -> Self {
        Engine {
            values: MapNodes::new(),
            options: options,
        }
    }

    pub fn handle_special_lines(&mut self, command: &str) {
        if self.options.verbosity > 0 {
            eprintln!("handle_special_lines: command = {}", command);
        }
        match command {
            "clear" => {
                self.values.nodes.clear();
            }

            "end" => {
                println!("{}", self.get_json());
                self.values.nodes.clear();
                exit(0);
            }

            "flush" => {
                println!("{}", self.get_json());
                self.values.nodes.clear();
                // exit(0);
            }

            _ => {
                eprintln!("unknown command: {}", command);
            }
        }
    }

    pub fn add_line(&mut self, line: &str) -> Result<()> {
        if self.options.verbosity > 0 {
            eprintln!("add line to engine: line = {}", line);
        }

        if line.starts_with(";") {
            let line = &line[1..];
            let parts = line.split_once(":");

            if let Some((dotted_key, value)) = parts {
                let value = value.trim();
                if dotted_key.starts_with("stdout.loop") {
                    self.handle_special_lines(value);
                    return Ok(());
                }

                let value = Node::Leaf(value.to_string().into());

                let keys: Vec<String> = dotted_key.split(".").map(|s| s.to_string()).collect();

                // let mut h = HashMap::new();

                let h = string_to_dict(keys, value);
                // self.values.extend(h);

                extend_hashmap(&mut self.values, h, &self.options)?;
            }

            // self.values.insert(dotted_key.to_string(), value);
        }
        Ok(())
    }

    /// Used for testing
    pub fn _get_json_object(&self) -> serde_json::Value {
        serde_json::to_value(&self.values).unwrap()
    }

    pub fn get_json(&self) -> String {
        let json = serde_json::to_string(&self.values).unwrap();
        json
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_string_to_dict() {
        let keys = vec!["a".to_string(), "b".to_string()];
        let value = Node::Leaf("1".to_string().into());
        let h = string_to_dict(keys, value);
        println!("h = {:?}", h);
    }

    #[test]
    fn test_empy_engine() -> anyhow::Result<()> {
        let mut engine = Engine::new(EngineOptions::new().with_verbosity(10));
        engine.add_line("line")?;
        let json = engine.get_json();
        assert_eq!(json, "{}");
        Ok(())
    }

    #[test]
    fn test_1_key_str() -> anyhow::Result<()> {
        let mut engine = Engine::new(EngineOptions::new().with_verbosity(10));
        engine.add_line(";a:value")?;
        let json = engine._get_json_object();
        let correct_json = json!({"a":"value"});
        assert_eq!(json, correct_json);
        Ok(())
    }

    #[test]
    fn test_1_key() -> anyhow::Result<()> {
        let mut engine = Engine::new(EngineOptions::new().with_verbosity(10));
        engine.add_line(";a:1")?;
        let json = engine._get_json_object();
        let correct_json = json!({"a":1});
        assert_eq!(json, correct_json);
        Ok(())
    }

    #[test]
    fn test_2_key() -> anyhow::Result<()> {
        let mut engine = Engine::new(EngineOptions::new().with_verbosity(10));
        engine.add_line(";a:1")?;
        engine.add_line(";b:2")?;
        let json = engine._get_json_object();
        let correct_json = json!({"a":1,"b":2});
        assert_eq!(json, correct_json);
        Ok(())
    }

    #[test]
    fn test_1_1_key() -> anyhow::Result<()> {
        let mut engine = Engine::new(EngineOptions::new().with_verbosity(10));
        engine.add_line(";a.b:1")?;
        let json = engine._get_json_object();
        let correct_json = json!({"a":{"b":1}});
        assert_eq!(json, correct_json);
        Ok(())
    }

    #[test]
    fn test_1_1_1_key() -> anyhow::Result<()> {
        let mut engine = Engine::new(EngineOptions::new().with_verbosity(10));
        engine.add_line(";a.b.c:1")?;
        let json = engine._get_json_object();
        let correct_json = json!({"a":{"b":{"c":1}}});
        assert_eq!(json, correct_json);
        Ok(())
    }

    #[test]
    fn test_1_1_1_key_bis() -> anyhow::Result<()> {
        let mut engine = Engine::new(EngineOptions::new().with_verbosity(10));
        engine.add_line(";a.b.c:1")?;
        let json = engine._get_json_object();
        let correct_json = json!({"a":{"b":{"c":1}}});
        assert_eq!(json, correct_json);
        Ok(())
    }

    #[test]
    fn test_1_2_key() -> anyhow::Result<()> {
        let mut engine = Engine::new(EngineOptions::new().with_verbosity(10));

        engine.add_line(";a:1")?;
        let json = engine._get_json_object();
        let correct_json = json!({"a":1});
        assert_eq!(json, correct_json);

        engine.add_line(";a.c:2")?;
        let json = engine._get_json_object();
        let correct_json = json!({
            "a":{"value": 1, "c": 2}     });
        assert_eq!(json, correct_json);
        Ok(())
    }

    #[test]
    fn test_1_1_2_key() -> anyhow::Result<()> {
        let mut engine = Engine::new(EngineOptions::new().with_verbosity(10));

        engine.add_line(";a.c:2")?;
        let json = engine._get_json_object();
        let correct_json = json!({
            "a":{"c": 2}     });
        assert_eq!(json, correct_json);

        engine.add_line(";a.b:3")?;
        let json = engine._get_json_object();
        let correct_json = json!({
            "a":{"c": 2, "b": 3}     });
        assert_eq!(json, correct_json);
        Ok(())
    }

    #[test]
    fn test_1_string_to_array_key() -> anyhow::Result<()> {
        let mut engine = Engine::new(EngineOptions::new().with_verbosity(10));

        engine.add_line(";a:2")?;
        let json = engine._get_json_object();
        let correct_json = json!({
            "a":2     });
        assert_eq!(json, correct_json);

        engine.add_line(";a:3")?;
        let json = engine._get_json_object();
        let correct_json = json!({
            "a": [2, 3]     });
        assert_eq!(json, correct_json);
        Ok(())
    }

    #[test]
    fn test_1_insert_string_to_array_key() -> anyhow::Result<()> {
        let mut engine = Engine::new(EngineOptions::new().with_verbosity(10));

        engine.add_line(";a:2")?;
        engine.add_line(";a:3")?;
        let json = engine._get_json_object();
        let correct_json = json!({
            "a": [2, 3]     });
        assert_eq!(json, correct_json);
        engine.add_line(";a:4")?;
        let json = engine._get_json_object();
        let correct_json = json!({
            "a": [2, 3, 4]     });
        assert_eq!(json, correct_json);
        Ok(())
    }

    #[test]
    fn test_double_dots_in_key() -> anyhow::Result<()> {
        let mut engine = Engine::new(EngineOptions::new().with_verbosity(10));

        engine.add_line(";a.b:2")?;
        let json = engine._get_json_object();
        let correct_json = json!({
            "a": {"b": 2}     });
        assert_eq!(json, correct_json);

        engine.add_line(";a.c:c:3")?;
        let json = engine._get_json_object();
        let correct_json = json!({
            "a": {"b": 2, "c": "c:3"}     });
        assert_eq!(json, correct_json);
        Ok(())
    }

    #[test]
    fn test_add_dict_in_array_err() -> anyhow::Result<()> {
        let mut engine = Engine::new(EngineOptions::new().with_verbosity(10));

        engine.add_line(";a:1")?;
        engine.add_line(";a:2")?;
        engine.add_line(";a:3")?;
        let json = engine._get_json_object();
        let correct_json = json!({
            "a": [1,2,3]     });
        assert_eq!(json, correct_json);

        let e = engine.add_line(";a.b:4");
        assert!(e.is_err());
        matches!(e, Err(Error::HowToDictInArray));
        let json = engine._get_json_object();
        let correct_json = json!(
            {"a":  [1, 2, 3]}
        );
        assert_eq!(json, correct_json);
        Ok(())
    }

    #[test]
    fn test_add_dict_in_array_merge() -> anyhow::Result<()> {
        let options = EngineOptions::new()
            .with_how_to_dict_in_array(HowToDictInArray::MergeDictInArray)
            .with_verbosity(10);
        let mut engine = Engine::new(options);

        engine.add_line(";a:1")?;
        engine.add_line(";a:2")?;
        engine.add_line(";a:3")?;
        let json = engine._get_json_object();
        let correct_json = json!({
            "a": [1,2,3]     });
        assert_eq!(json, correct_json);

        let e = engine.add_line(";a.b:4");
        assert!(e.is_ok());
        let json = engine._get_json_object();
        let correct_json = json!(
            {"a":  [1, 2, 3, {"b" : 4}]}
        );
        assert_eq!(json, correct_json);

        let e = engine.add_line(";a.b:5");
        assert!(e.is_ok());
        let json = engine._get_json_object();
        let correct_json = json!(
            {"a":  [1, 2, 3, {"b" : 4}, {"b" : 5}]}
        );
        assert_eq!(json, correct_json);
        Ok(())
    }
}
