Picture the scene. You have a massive array of JSON objects, and you want to know what it looks like. You're going through, CTRL + F'ing the F'ing blob, but some keys have different types. Some are null in one object but a string in another. Some aren't even there at all! You're left thinking; "what the fuck is this JSON?!".

`what-the-fuck-is-this-json` tells you the schema of an object array. It can read from files or even accept an endpoint url. It can generate types and structs, ready to drop into your code. It alpabetically rearranges the keys. It can do a dance to bring the rain (not really).

## Usage
`[what-the-fuck-is-this-json | wut] [OPTIONS] <FILE_PATH | URL>`

(`what-the-fuck-is-this-json` is aliased as `wut`)

### Struct definition generation

For demonstration purposes, `test.json` contains the following;
```json
[{"id":1, "foo": "food?", "bar": "in walks a horse"}, {"id": 2, "foo": null}, {"id": 3, "foo": "foood!", "bar": 123}]
```


#### Basic
`wut test.json`
```text
bar : String, Absent, Integer
foo : Null, String
id  : Integer
```

#### Rust
`wut test.json -r`
```rust
#[derive(Deserialize, Serialize, Debug)]
struct MyStruct {
	// bar ∈ {Absent, Integer, String}
	foo: Option<String>, // Can be null
	id: i64
}
```

### Fin

For more help, run `what-the-fuck-is-this-json --help`, or file an issue.
