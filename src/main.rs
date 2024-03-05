use std::fs;
use std::collections::hash_map;
use std::fmt::Display;

#[derive(Debug, Clone)]
enum Keyword {
    Let,
    Print,
    PrintLn,
    Fn,
    For,
    If
}

#[derive(Debug, Clone)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Invert,
    TupleStart,
    TupleEnd,
    BlockStart,
    BlockEnd,
    ArrayStart,
    ArrayEnd,
    CallFn,
    IndexArray
}

#[derive(Debug, Clone)]
struct Fn {
    args: Vec<String>,
    body: Vec<Value>,
}

#[derive(Debug, Clone)]
enum Value {
    Int(i32),
    String(String),
    Ident(String),
    Operation(Op),
    Keyword(Keyword),
    Fn(Fn),
    Tuple(Vec<Value>),
    Block(Vec<Value>),
    Array(Vec<Value>),
    None
}

// type TypeRef = usize;

// #[derive(Debug, Clone)]
// enum TypeType {
//     Int(bool),
//     Float,
//     Array(TypeRef),
// }

// #[derive(Debug, Clone)]
// struct TypeInfo {
//     size: i32,
//     typ: TypeType,
// }

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Ident(i) => {
                write!(f, "(ident: {})", i)
            }
            Value::Int(i) => {
                write!(f, "{}", i)
            }
            Value::Keyword(kw) => {
                write!(f, "{:?}", kw)
            }
            Value::String(s) => {
                write!(f, "{}", s)
            }
            Value::None => {
                write!(f, "none")
            }
            Value::Operation(op) => {
                write!(f, "(op: {:?})", op)
            }
            Value::Fn(f_) => {
                write!(f, "(fn: {:?})", f_)
            }
            Value::Tuple(t) => {
                write!(f, "(").unwrap();
                for (i, v) in t.iter().enumerate() {
                    write!(f, "{}", v).unwrap();
                    if i != t.len() - 1 {
                        write!(f, " ").unwrap();
                    }
                }
                write!(f, ")")
            }
            Value::Block(b) => {
                write!(f, "{{\n\t").unwrap();
                for (i, v) in b.iter().enumerate() {
                    write!(f, "{}", v).unwrap();
                    if i != b.len() - 1 {
                        write!(f, " ").unwrap();
                    } else {
                        write!(f, "\n").unwrap();
                    }
                }
                write!(f, "}}")
            }
            Value::Array(b) => {
                write!(f, "[\n\t").unwrap();
                for (i, v) in b.iter().enumerate() {
                    write!(f, "{}", v).unwrap();
                    if i != b.len() - 1 {
                        write!(f, " ").unwrap();
                    } else {
                        write!(f, "\n").unwrap();
                    }
                }
                write!(f, "]")
            }
        }
    }
}

#[derive(Debug)]
enum Delim {
    Tuple(Vec<Value>),
    Block(Vec<Value>),
    Array(Vec<Value>)
}

#[derive(Debug)]
struct InterpreterState {
    stack: Vec<Value>,
    vars: hash_map::HashMap<String, Value>,
    delims: Vec<Delim>
}

impl InterpreterState {
    fn get_int(&mut self) -> Option<i32> {
        let val = self.stack.pop().unwrap();
        match val {
            Value::Int(i) => {
                return Some(i);
            }
            Value::Ident(ref i) => {
                if let Some(Value::Int(v)) = self.vars.get(i) {
                    return Some(*v);
                } else {
                    return None;
                }
            }
            _ => {
                return None;
            }
        }
    }
    fn push_value(&mut self, val: Value) {
        self.stack.push(val);
    }
    fn pop_value(&mut self) -> Option<Value> {
        self.stack.pop()
    }
    fn get_value(&mut self) -> Option<Value> {
        let v = self.pop_value();
        if let Some(Value::Ident(ref i)) = v {
            let r = self.get_var(i);
            if r.is_some() {
                return r.cloned();
            }
        }
        return v;
    }
    fn add_var(&mut self, name: &str) {
        self.vars.insert(name.to_string(), Value::None);
    }
    fn set_var(&mut self, name: &str, val: Value) {
        let chud = self.vars.get_mut(name).unwrap();
        *chud = val;
    }
    fn get_var(&mut self, name: &str) -> Option<&Value> {
        self.vars.get(name)
    }
    fn eval_tuple(&mut self, tuple: Value) -> Value {
        if let Value::Tuple(t) = tuple {
            let mut istate_new = InterpreterState {
                stack: Vec::new(),
                vars: self.vars.clone(),
                delims: Vec::new()
            };
            run_interp(&mut istate_new, &t);
            return Value::Tuple(istate_new.stack);
        } else {
            return tuple;
        }
    }
    fn eval_array(&mut self, tuple: Value) -> Value {
        if let Value::Array(t) = tuple {
            let mut istate_new = InterpreterState {
                stack: Vec::new(),
                vars: self.vars.clone(),
                delims: Vec::new()
            };
            run_interp(&mut istate_new, &t);
            return Value::Array(istate_new.stack);
        } else {
            return tuple;
        }
    }
}

fn run_interp(istate: &mut InterpreterState, vals: &[Value]) {
    for val in vals {
        if !istate.delims.is_empty() {
            match istate.delims.last_mut().unwrap() {
                Delim::Block(vs) => {
                    if let Value::Operation(Op::BlockEnd) = val {
                        if let Delim::Block(b) = istate.delims.pop().unwrap() {
                            istate.push_value(Value::Block(b));
                        } else {
                            println!("{:?}", istate);
                            panic!("cant end non-block with block end");
                        }
                    } else {
                        vs.push(val.clone());
                    }
                }
                Delim::Tuple(vs) => {
                    if let Value::Operation(Op::TupleEnd) = val {
                        if let Delim::Tuple(t) = istate.delims.pop().unwrap() {
                            istate.push_value(Value::Tuple(t));
                        } else {
                            println!("{:?}", istate);
                            panic!("cant end non-tuple with tuple end");
                        }
                    } else {
                        vs.push(val.clone());
                    }
                }
                Delim::Array(vs) => {
                    if let Value::Operation(Op::ArrayEnd) = val {
                        if let Delim::Array(t) = istate.delims.pop().unwrap() {
                            let chud = istate.eval_array(Value::Array(t));
                            istate.push_value(chud);
                        } else {
                            println!("{:?}", istate);
                            panic!("cant end non-tuple with tuple end");
                        }
                    } else {
                        vs.push(val.clone());
                    }
                }
            }
            continue;
        }
        match val {
            Value::Operation(op) => {
                match op {
                    Op::Assign => {
                        let v = istate.get_value().unwrap();
                        if let Value::Ident(k) = istate.stack.pop().unwrap() {
                            istate.set_var(&k, v.clone());
                            // println!("set var {} to value {:?}", &k, v);
                        } else {
                            println!("{:?}", istate);
                            panic!("type mismatch");
                        }
                    }
                    Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Mod => {
                        let b = istate.get_int().unwrap();
                        let a = istate.get_int().unwrap();
                        match op {
                            Op::Add => {
                                istate.push_value(Value::Int(a + b));
                            }
                            Op::Sub => {
                                istate.push_value(Value::Int(a - b));
                            }
                            Op::Mul => {
                                istate.push_value(Value::Int(a * b));
                            }
                            Op::Div => {
                                istate.push_value(Value::Int(a / b));
                            }
                            Op::Mod => {
                                istate.push_value(Value::Int(a % b));
                            }
                            _ => {}
                        }
                    }
                    Op::Invert => {
                        let a = istate.get_int().unwrap();
                        istate.push_value(Value::Int(if a != 0 { 0 } else { 1 }));
                    }
                    Op::BlockStart => {
                        istate.delims.push(Delim::Block(Vec::new()));
                    }
                    Op::TupleStart => {
                        istate.delims.push(Delim::Tuple(Vec::new()));
                    }
                    Op::ArrayStart => {
                        istate.delims.push(Delim::Array(Vec::new()));
                    }
                    Op::CallFn => {
                        if let Value::Fn(f) = istate.get_value().unwrap() {
                            let mut new_istate = InterpreterState {
                                stack: Vec::new(),
                                vars: hash_map::HashMap::new(),
                                delims: Vec::new(),
                            };
                            for arg in f.args.iter().rev() {
                                new_istate.add_var(&arg);
                                new_istate.set_var(&arg, istate.get_value().unwrap());
                            }
                            run_interp(&mut new_istate, &f.body);
                        } else {
                            println!("{:?}", istate);
                            panic!("cant call non-fn");
                        }
                    }
                    Op::IndexArray => {
                        let index = istate.get_int().unwrap();
                        let array = istate.get_value().unwrap();
                        if let Value::Array(a) = array {
                            istate.push_value(a[index as usize].clone());
                        } else {
                            println!("{:?}", istate);
                            panic!("index an array you tard");
                        }
                    }
                    _ => {}
                }
            }
            Value::Int(_) => {
                istate.push_value(val.clone());
            }
            Value::String(_) => {
                istate.push_value(val.clone());
            }
            Value::Ident(_) => {
                istate.push_value(val.clone());
            }
            Value::Fn(_) => {
                istate.push_value(val.clone());
            }
            Value::Keyword(ref kw) => {
                match kw {
                    Keyword::Let => {
                        if let Value::Ident(i) = istate.stack.pop().unwrap() {
                            istate.add_var(&i);
                            // println!("added var {}", &i);
                            istate.push_value(Value::Ident(i));
                        } else {
                            println!("{:?}", istate);
                            panic!("use let on an ident, dummy!");
                        }
                    }
                    Keyword::Fn => {
                        let block_ = istate.get_value().unwrap();
                        let tuple_ = istate.get_value().unwrap();
                        if let Value::Block(block) = block_ {
                            if let Value::Tuple(tuple) = tuple_ {
                                let mut args = vec![];
                                for arg in tuple {
                                    if let Value::Ident(i) = arg {
                                        args.push(i);
                                    }
                                }
                                istate.push_value(Value::Fn(Fn { args, body: block }));
                            } else {
                                println!("{:?}", istate);
                                panic!("try to create a function properly next time");
                            }
                        } else {
                            println!("{:?}", istate);
                            panic!("try to create a function properly next time");
                        }
                    }
                    Keyword::Print => {
                        let v = istate.get_value().unwrap();
                        print!("{}", istate.eval_tuple(v));
                    }
                    Keyword::PrintLn => {
                        let v = istate.get_value().unwrap();
                        println!("{}", istate.eval_tuple(v));
                    }
                    Keyword::For => {
                        let block = istate.get_value().unwrap();
                        let val_name = istate.pop_value().unwrap();
                        let mut array = istate.get_value().unwrap();
                        array = istate.eval_array(array); // TODO remove unnecessary eval when its not a literal
                        let mut istate_new = InterpreterState {
                            stack: Vec::new(),
                            vars: istate.vars.to_owned(),
                            delims: Vec::new()
                        };
                        if let Value::Array(a) = array {
                            if let Value::Ident(ref i) = val_name {
                                if let Value::Block(ref b) = block {
                                    istate_new.add_var(i);
                                    for val in a {
                                        istate_new.set_var(i, val);
                                        run_interp(&mut istate_new, b);
                                    }
                                    for var in istate.vars.iter_mut() {
                                        *var.1 = istate_new.get_var(var.0).unwrap().clone();
                                    }
                                } else {
                                    println!("{:?}", istate);
                                    panic!("not a block {:?}", block);
                                }
                            } else {
                                println!("{:?}", istate);
                                panic!("not an ident {:?}", val_name);
                            }
                        } else {
                            println!("{:?}", istate);
                            panic!("not an array {:?}", array);
                        }
                    }
                    Keyword::If => {
                        let block = istate.get_value().unwrap();
                        let cond = istate.get_int().unwrap();
                        if cond != 0 {
                            if let Value::Block(ref b) = block {
                                let mut istate_new = InterpreterState {
                                    stack: Vec::new(),
                                    vars: istate.vars.to_owned(),
                                    delims: Vec::new()
                                };
                                run_interp(&mut istate_new, b);
                                for var in istate.vars.iter_mut() {
                                    *var.1 = istate_new.get_var(var.0).unwrap().clone();
                                }
                            } else {
                                println!("{:?}", istate);
                                panic!("not a block {:?}", block);
                            }
                        }
                    }
                }
            }
            Value::Tuple(_) => {}
            Value::Block(_) => {}
            Value::Array(_) => {}
            Value::None => {}
        }
    }
}

fn main() {
    let fortnite = fs::read_to_string("knusper_chud").unwrap();
    let mut cur_val = Value::None;
    let mut cur_str = String::new();
    let mut vals = vec![];
    for ch in fortnite.chars() {
        match cur_val {
            Value::None => {
                if ch.is_numeric() {
                    cur_val = Value::Int(0);
                    cur_str.push(ch);
                } else if ch.is_ascii_alphabetic() {
                    cur_val = Value::Ident(String::new());
                    cur_str.push(ch);
                } else if ch == '"' {
                    cur_val = Value::String(String::new());
                    // cur_str.push(ch);
                } else if ch == ' ' || ch == '\n' {
                    cur_str.clear();
                } else {
                    let op =
                        match ch {
                            '+' => {Op::Add}
                            '-' => {Op::Sub}
                            '*' => {Op::Mul}
                            '/' => {Op::Div}
                            '%' => {Op::Mod}
                            '=' => {Op::Assign}
                            '!' => {Op::Invert}
                            '(' => {Op::TupleStart}
                            ')' => {Op::TupleEnd}
                            '{' => {Op::BlockStart}
                            '}' => {Op::BlockEnd}
                            '[' => {Op::ArrayStart}
                            ']' => {Op::ArrayEnd}
                            '@' => {Op::CallFn}
                            '#' => {Op::IndexArray}
                            _ => {panic!("invalid char {}", ch)}
                        };
                    cur_val = Value::Operation(op);
                }
            }
            Value::Int(_) => {
                if !ch.is_numeric() {
                    vals.push(Value::Int(cur_str.parse().unwrap()));
                    cur_str.clear();
                    cur_val = Value::None;
                    continue;
                }
                cur_str.push(ch);
            }
            Value::String(_) => {
                if ch == '"' {
                    vals.push(Value::String(cur_str.clone()));
                    cur_str.clear();
                    cur_val = Value::None;
                    continue;
                }
                cur_str.push(ch);
            }
            Value::Ident(_) => {
                if !ch.is_alphanumeric() {
                    match cur_str.as_str() {
                        "let" => {
                            vals.push(Value::Keyword(Keyword::Let));
                        }
                        "print" => {
                            vals.push(Value::Keyword(Keyword::Print));
                        }
                        "println" => {
                            vals.push(Value::Keyword(Keyword::PrintLn));
                        }
                        "fn" => {
                            vals.push(Value::Keyword(Keyword::Fn));
                        }
                        "for" => {
                            vals.push(Value::Keyword(Keyword::For));
                        }
                        "if" => {
                            vals.push(Value::Keyword(Keyword::If));
                        }
                        _ => {
                            vals.push(Value::Ident(cur_str.clone()));
                        }
                    }
                    cur_str.clear();
                    cur_val = Value::None;
                    continue;
                }
                cur_str.push(ch);
            }
            Value::Operation(ref cop) => {
                let op = match ch {
                    '=' => {
                        match cop {
                            Op::Add => {
                                Op::AddAssign
                            }
                            Op::Sub => {
                                Op::SubAssign
                            }
                            Op::Mul => {
                                Op::MulAssign
                            }
                            Op::Div => {
                                Op::DivAssign
                            }
                            _ => {
                                panic!("invalid operator");
                            }
                        }
                    }
                    _ => {
                        vals.push(cur_val);
                        cur_str.clear();
                        cur_val = Value::None;
                        continue;
                    }
                };
                cur_val = Value::Operation(op);
            }
            _ => {}
        }
    }
    // println!("Hello, world! {:?}", vals);
    let mut istate = InterpreterState {
        stack: vec![],
        vars: hash_map::HashMap::new(),
        delims: Vec::new()
    };
    run_interp(&mut istate, &vals);
    // println!("{:?}, {:?}", istate.stack, istate.vars);
}
