use std::fs;
use std::collections::hash_map;
use std::fmt::Display;
use knusper::InterpreterState;
use knusper::Value;
use knusper::tokenize;

fn main() {
    //if let Some(file) = std::env::args().skip(1).next() {
    let file = "knusper_chud";
        let fortnite = fs::read_to_string(file).unwrap();
        // println!("Hello, world! {:?}", vals);
        let mut ext_fns: hash_map::HashMap<String, fn(Value) -> Value> = hash_map::HashMap::new();
        ext_fns.insert("joe".to_string(), | a: Value | {
            println!("the joe biden among us drip shirt");
            Value::None
        });
        let mut istate = InterpreterState {
            stack: vec![],
            vars: hash_map::HashMap::new(),
            globals: hash_map::HashMap::new(),
            delims: Vec::new(),
            ext_fns: &ext_fns,
        };
        let vals = tokenize(&fortnite);
        istate.run(&vals);
        // println!("{:?}, {:?}", istate.stack, istate.vars);
    //} else {
    //    println!("argument required");
    //}
}
