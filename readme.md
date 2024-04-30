# cout2json

Cout2json is a utility that generates JSON structures from cout output. In a program, you construct the JSON object by printing key values in the terminal.

Example: 

```bash
printf ";key:value" | cout2json # give {"key":"value"}
printf ";key1:value1\n;key2:value2" | cout2json # give {"key1":"value1","key2":"value2"} 
printf ";a.b.c:value" | cout2json # give {"a":{"b":{"c":"value"}}}
```

The output can be piped to tools like jq to further process the output. 

```bash
printf ";key1.b:1\n;key2.d:2" | cout2json | jq ".key2" -c # give {"d":2}
```

## Use Cases

- Instrument a library with minimal code alteration, primarily relying on printing to cout.
- Given that the JSON structure is constructed externally to the program, dictionary keys can be universally utilized throughout the code. Cout2json is responsible for generating a cohesive structure.
- Applicable to microcontrollers capable of serial printing. Eliminates the necessity of managing JSON structures within the microcontroller.
- 
## Install 

```bash
cargo install cout2json
```

## Features

### Automatically build nested structure when key is separated by a period.
```bash
printf ";key1.key2:1" | cout2json               # give {"key1":{"key2":1}}
printf ";key1.key2.key3:1" | cout2json          # give {"key1":{"key2":{"key3":1}}}
printf ";key1.key2:1\n;key1.key3:2" | cout2json # give {"key1":{"key2":1,"key3":2}}
```

### Automatically interpret values as integers, floats, or strings.

```bash
printf ";key:1" | cout2json     # give {"key":1}
printf ";key:1.0" | cout2json   # give {"key":1.0}
printf ";key:a" | cout2json     # give {"key":"a"}
```

### Automatically build array when same key is repeated.

```bash
printf ";key:1\n;key:2" | cout2json       # give {"key":[1,2]}
printf ";key.a:1\n;key.b:2" | cout2json   # give {"key":{"a":1,"b":2}}
```


