pub fn to_postgresql_type(json_type: &str)-> String {

    return match json_type {
        "text" => "TEXT".to_string(),
        "date" => "TIMESTAMP".to_string(),
        "number" => "NUMERIC".to_string(),
        "boolean" => "BOOLEAN".to_string(),
        _ => "TEXT".to_string()
    }

}
