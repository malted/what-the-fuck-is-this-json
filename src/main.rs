use clap::Parser;
use serde_json::{from_reader, Value};
use std::collections::{HashMap, HashSet};
use std::fs::File;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The input source URL or file
    #[arg(required = true)]
    input_source: String,

    /// Generate Rust structs from the input instead of a basic output
    #[arg(short, long, default_value_t = false)]
    rs_struct_generate: bool,

    /// Include comments in the Rust structs
    #[arg(short, long, default_value_t = true)]
    with_comments: bool,
}

fn main() {
    let args = Args::parse();

    let json: Vec<Value> = match reqwest::Url::parse(&args.input_source) {
        Ok(url) => reqwest::blocking::get(url).unwrap().json().unwrap(),
        Err(_) => from_reader(File::open(args.input_source).unwrap())
            .expect("the input should be an array of objects"),
    };

    let keys: HashSet<String> = json
        .iter()
        .flat_map(|x| x.as_object().unwrap().keys().cloned())
        .collect();

    #[derive(Debug, PartialEq, Eq, Hash)]
    enum ValueType {
        Null,
        Bool,
        Integer,
        Float,
        String,
        Array,
        Object,
        Absent,
    }
    impl ValueType {
        fn as_rust_type(&self) -> &'static str {
            match self {
                ValueType::Null => "Option<serde_json::Value>",
                ValueType::Bool => "bool",
                ValueType::Integer => "i64",
                ValueType::Float => "f64",
                ValueType::String => "String",
                ValueType::Array => "Vec<serde_json::Value>",
                ValueType::Object => "serde_json::Value",
                ValueType::Absent => "Option<serde_json::Value>",
            }
        }
    }

    let mut results: HashMap<String, HashSet<ValueType>> = HashMap::new();

    for x in json {
        // The keys that are not present in x
        let difference: HashSet<_> = keys
            .difference(&x.as_object().unwrap().keys().cloned().collect())
            .cloned()
            .collect();

        for key in difference {
            results.entry(key).or_default().insert(ValueType::Absent);
        }

        // Now we can just iterate over the keys in x and insert the value types
        for (key, value) in x.as_object().unwrap() {
            let value_type = match value {
                Value::Null => ValueType::Null,
                Value::Bool(_) => ValueType::Bool,
                Value::Number(_) => {
                    if value.as_i64().is_some() || value.as_u64().is_some() {
                        ValueType::Integer
                    } else if value.as_f64().is_some() {
                        ValueType::Float
                    } else {
                        ValueType::String
                    }
                }
                Value::String(_) => ValueType::String,
                Value::Array(_) => ValueType::Array,
                Value::Object(_) => ValueType::Object,
            };

            results.entry(key.clone()).or_default().insert(value_type);
        }
    }

    // Sort keys and determine the width for formatting
    let mut sorted_keys: Vec<_> = results.keys().collect();
    sorted_keys.sort();
    let max_width = sorted_keys.iter().map(|k| k.len()).max().unwrap_or(0);

    let basic = !args.rs_struct_generate && !args.rs_struct_generate;

    if basic {
        for key in sorted_keys.clone() {
            let vals_fmt = results[key]
                .iter()
                .map(|x| format!("{:?}", x))
                .collect::<Vec<_>>()
                .join(", ");

            println!("{:<width$} : {}", key, vals_fmt, width = max_width);
        }
    } else if args.rs_struct_generate {
        println!("#[derive(Deserialize, Serialize, Debug)]");
        println!("struct MyStruct {{");
        for key in sorted_keys.clone() {
            let rust_formatted_key = key.replace("-", "_");
            let r: &HashSet<ValueType> = results.get(key).unwrap();
            // r but removed Null and Absent, and as a string
            let r_removed = r
                .iter()
                .filter(|x| **x != ValueType::Null && **x != ValueType::Absent)
                .map(|x| format!("{}", x.as_rust_type()))
                .collect::<Vec<_>>();

            if r_removed.len() != 1 && args.with_comments {
                println!("\t// {rust_formatted_key} ∈ {:?}", r);
                continue;
            }

            let r_fmt = r_removed.join(" | ");

            print!("\t{}: ", rust_formatted_key);
            if r.contains(&ValueType::Null) || r.contains(&ValueType::Absent) {
                print!("Option<{r_fmt}>");
            } else {
                print!("{r_fmt}");
            }

            let absent_or_null = if r.contains(&ValueType::Null) && r.contains(&ValueType::Absent) {
                "// Can be absent or null"
            } else if r.contains(&ValueType::Null) {
                "// Can be null"
            } else if r.contains(&ValueType::Absent) {
                "// Can be absent"
            } else {
                ""
            };

            // If last key, don't print a comma
            if key.as_str() != sorted_keys.last().unwrap().as_str() {
                print!(",");
            }

            if args.with_comments {
                print!(" {absent_or_null}");
            }

            println!();
        }
    }

    if args.rs_struct_generate {
        println!("}}");
    }
}
