use jsonref::JsonRef;
use std::error::Error;
use serde_json::Value;
use std::collections::HashMap;


#[derive(Clone, Debug, PartialEq)]
struct SchemaOrder {
    schema: String,
    output: Vec<String>,
}

impl SchemaOrder {

    fn new(schema: &str) -> SchemaOrder {
        return SchemaOrder {
            schema: schema.to_owned(),
            output: vec![],
        }
    }

    fn parse(&mut self) -> Result<(), Box<dyn Error + Sync + Send>> {
        let mut jsonref = JsonRef::new();
        let value: Value;
        if self.schema.starts_with("http") {
            value = jsonref.deref_url(&self.schema)?;
        } else {
            value = jsonref.deref_file(&self.schema)?;
        }

        self.parse_value(value);

        Ok(())
    }

    fn parse_value(&mut self, schema: Value) {
        if let Some(object) = schema.get("properties") {
            self.parse_properties(object, vec![]);
        }
    }

    fn parse_properties(&mut self, properties: &Value, path: Vec<String>) {

        if let Some(obj) = properties.as_object() {
            for (name, property) in obj {

                let mut new_path = path.clone();
                new_path.push(name.clone());

                if let Some(properties) = property.get("properties") {
                    self.parse_properties(properties, new_path.clone());
                } else if let Some(properties) = property.pointer("/items/properties") {
                    self.parse_properties(properties, new_path.clone());
                } else {
                    let field_path = new_path.join("_");
                    self.output.push(field_path);
                }
            }
        }
    }
}

pub fn schema_order(schema_path: &str) -> Result<HashMap<String, usize>, Box<dyn Error + Sync + Send>>{
    let mut schema = SchemaOrder::new(schema_path);
    schema.parse()?;

    let mut output_map = HashMap::new();

    for (num, item) in schema.output.iter().enumerate() {
        output_map.insert(item.clone(), num + 1);
    }
    return Ok(output_map)
}


#[cfg(test)]
mod tests {
    use super::*;
    //use serde_json::json;

    #[test]
    fn test_remote_schema() {

        let expected: HashMap<String, usize> = HashMap::from([
          ("prop1".to_string(), 1),
          ("prop2".to_string(), 2),
          ("prop3_prop1".to_string(), 3),
          ("prop3_prop2".to_string(), 4),
          ("prop4".to_string(), 5),
          ("prop5".to_string(), 6),
        ]);


        assert_eq!(
            schema_order(
                "https://gist.githubusercontent.com/kindly/91e09f88ced65aaca1a15d85a56a28f9/raw/52f8477435cff0b73c54aacc70926c101ce6c685/base.json"
                ).unwrap(), expected
            );
    }
}
