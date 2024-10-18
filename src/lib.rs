use std::fs;
use std::collections::hash_map;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Keyword {
    Let,
    Global,
    Print,
    PrintLn,
    Fn,
    For,
    If
}

#[derive(Debug, Clone)]
pub enum Op {
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
    IndexArray,
}

#[derive(Debug, Clone)]
pub struct Fn {
    args: Vec<String>,
    body: Vec<Value>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Char(char),
    String(String),
    Ident(String),
    ExtFn(String),
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
            Value::ExtFn(i) => {
                write!(f, "(ext_fn: {})", i)
            }
            Value::Int(i) => {
                write!(f, "{}", i)
            }
            Value::Char(c) => {
                write!(f, "{}", c)
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
pub enum Delim {
    Tuple(Vec<Value>),
    Block(Vec<Value>),
    Array(Vec<Value>)
}

#[derive(Debug)]
pub struct InterpreterState<'a> {
    pub stack: Vec<Value>,
    pub vars: hash_map::HashMap<String, Value>,
    pub globals: hash_map::HashMap<String, Value>,
    pub delims: Vec<Delim>,
    pub ext_fns: &'a hash_map::HashMap<String, fn(Value) -> Value>
}

impl<'a> InterpreterState<'a> {
    fn get_int(&mut self) -> Option<i32> {
        let val = self.stack.pop().unwrap();
        match val {
            Value::Int(i) => {
                return Some(i);
            }
            Value::Ident(ref i) => {
                if let Some(Value::Int(v)) = self.get_var(i) {
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
            } else if self.ext_fns.contains_key(i) {
                return Some(Value::ExtFn(i.to_string()));
            }
        }
        return v;
    }
    fn add_global(&mut self, name: &str) {
        self.globals.insert(name.to_string(), Value::None);
    }
    fn add_var(&mut self, name: &str) {
        self.vars.insert(name.to_string(), Value::None);
    }
    fn set_var(&mut self, name: &str, val: Value) {
        let chud = self.vars.get_mut(name)
            .or(self.globals.get_mut(name))
            .unwrap();
        *chud = val;
    }
    fn get_var(&mut self, name: &str) -> Option<&Value> {
        self.vars.get(name)
            .or(self.globals.get(name))
    }
    fn eval_tuple(&mut self, tuple: Value) -> Value {
        if let Value::Tuple(t) = tuple {
            let mut istate_new = InterpreterState {
                stack: Vec::new(),
                vars: self.vars.clone(),
                globals: self.globals.clone(),
                delims: Vec::new(),
                ext_fns: self.ext_fns
            };
            istate_new.run(&t);
            self.globals = istate_new.globals;
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
                globals: self.globals.clone(),
                delims: Vec::new(),
                ext_fns: self.ext_fns
            };
            istate_new.run(&t);
            self.globals = istate_new.globals;
            return Value::Array(istate_new.stack);
        } else {
            return tuple;
        }
    }
    pub fn run(&mut self, vals: &[Value]) {
        for val in vals {
            if !self.delims.is_empty() {
                match self.delims.last_mut().unwrap() {
                    Delim::Block(vs) => {
                        if let Value::Operation(Op::BlockEnd) = val {
                            if let Delim::Block(b) = self.delims.pop().unwrap() {
                                self.push_value(Value::Block(b));
                            } else {
                                println!("{:?}", self);
                                panic!("cant end non-block with block end");
                            }
                        } else {
                            vs.push(val.clone());
                        }
                    }
                    Delim::Tuple(vs) => {
                        if let Value::Operation(Op::TupleEnd) = val {
                            if let Delim::Tuple(t) = self.delims.pop().unwrap() {
                                self.push_value(Value::Tuple(t));
                            } else {
                                println!("{:?}", self);
                                panic!("cant end non-tuple with tuple end");
                            }
                        } else {
                            vs.push(val.clone());
                        }
                    }
                    Delim::Array(vs) => {
                        if let Value::Operation(Op::ArrayEnd) = val {
                            if let Delim::Array(t) = self.delims.pop().unwrap() {
                                let chud = self.eval_array(Value::Array(t));
                                self.push_value(chud);
                            } else {
                                println!("{:?}", self);
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
                            let v = self.get_value().unwrap();
                            if let Value::Ident(k) = self.stack.pop().unwrap() {
                                self.set_var(&k, v.clone());
                                // println!("set var {} to value {:?}", &k, v);
                            } else {
                                println!("{:?}", self);
                                panic!("type mismatch");
                            }
                        }
                        Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Mod => {
                            let b = self.get_int().unwrap();
                            let a = self.get_int().unwrap();
                            match op {
                                Op::Add => {
                                    self.push_value(Value::Int(a + b));
                                }
                                Op::Sub => {
                                    self.push_value(Value::Int(a - b));
                                }
                                Op::Mul => {
                                    self.push_value(Value::Int(a * b));
                                }
                                Op::Div => {
                                    self.push_value(Value::Int(a / b));
                                }
                                Op::Mod => {
                                    self.push_value(Value::Int(a % b));
                                }
                                _ => {}
                            }
                        }
                        Op::Invert => {
                            let a = self.get_int().unwrap();
                            self.push_value(Value::Int(if a != 0 { 0 } else { 1 }));
                        }
                        Op::BlockStart => {
                            self.delims.push(Delim::Block(Vec::new()));
                        }
                        Op::TupleStart => {
                            self.delims.push(Delim::Tuple(Vec::new()));
                        }
                        Op::ArrayStart => {
                            self.delims.push(Delim::Array(Vec::new()));
                        }
                        Op::CallFn => {
                            match self.get_value().unwrap() {
                                Value::Fn(f) => {
                                    let mut istate_new = InterpreterState {
                                        stack: Vec::new(),
                                        vars: hash_map::HashMap::new(),
                                        globals: self.globals.clone(),
                                        delims: Vec::new(),
                                        ext_fns: self.ext_fns
                                    };
                                    for arg in f.args.iter().rev() {
                                        istate_new.add_var(&arg);
                                        istate_new.set_var(&arg, self.get_value().unwrap());
                                    }
                                    istate_new.run(&f.body);
                                    self.globals = istate_new.globals;
                                }
                                // TODO improvements needed
                                Value::ExtFn(ref _f) => {
                                    let f = self.ext_fns.get(_f).unwrap();
                                    let _val = self.get_value();
                                    let val = if _val.is_none() {
                                        Value::None
                                    } else {
                                        _val.unwrap()
                                    };
                                    let res = f(val);
                                    self.push_value(res);
                                }
                                _ => {
                                    println!("{:?}", self);
                                    panic!("cant call non-fn");
                                }
                            }
                        }
                        Op::IndexArray => {
                            let index = self.get_int().unwrap();
                            let array = self.get_value().unwrap();
                            if let Value::Array(a) = array {
                                self.push_value(a[index as usize].clone());
                            } else if let Value::String(a) = array {
                                self.push_value(Value::Char(a.as_bytes()[index as usize].into()));
                            } else {
                                println!("{:?}", self);
                                panic!("index an array you tard");
                            }
                        }
                        _ => {}
                    }
                }
                Value::Int(_) => {
                    self.push_value(val.clone());
                }
                Value::Char(_) => {
                    self.push_value(val.clone());
                }
                Value::String(_) => {
                    self.push_value(val.clone());
                }
                Value::Ident(_) => {
                    self.push_value(val.clone());
                }
                Value::Fn(_) => {
                    self.push_value(val.clone());
                }
                Value::ExtFn(_) => {
                    self.push_value(val.clone());
                }
                Value::Keyword(ref kw) => {
                    match kw {
                        Keyword::Let => {
                            if let Value::Ident(i) = self.stack.pop().unwrap() {
                                self.add_var(&i);
                                // println!("added var {}", &i);
                                self.push_value(Value::Ident(i));
                            } else {
                                println!("{:?}", self);
                                panic!("use let on an ident, dummy!");
                            }
                        }
                        Keyword::Global => {
                            if let Value::Ident(i) = self.stack.pop().unwrap() {
                                self.add_global(&i);
                                // println!("added var {}", &i);
                                self.push_value(Value::Ident(i));
                            } else {
                                println!("{:?}", self);
                                panic!("use let on an ident, dummy!");
                            }
                        }
                        Keyword::Fn => {
                            let block_ = self.get_value().unwrap();
                            let tuple_ = self.get_value().unwrap();
                            if let Value::Block(block) = block_ {
                                if let Value::Tuple(tuple) = tuple_ {
                                    let mut args = vec![];
                                    for arg in tuple {
                                        if let Value::Ident(i) = arg {
                                            args.push(i);
                                        }
                                    }
                                    self.push_value(Value::Fn(Fn { args, body: block }));
                                } else {
                                    println!("{:?}", self);
                                    panic!("try to create a function properly next time");
                                }
                            } else {
                                println!("{:?}", self);
                                panic!("try to create a function properly next time");
                            }
                        }
                        Keyword::Print => {
                            let v = self.get_value().unwrap();
                            print!("{}", self.eval_tuple(v));
                        }
                        Keyword::PrintLn => {
                            let v = self.get_value().unwrap();
                            println!("{}", self.eval_tuple(v));
                        }
                        Keyword::For => {
                            let block = self.get_value().unwrap();
                            let val_name = self.pop_value().unwrap();
                            let mut array = self.get_value().unwrap();
                            array = self.eval_array(array); // TODO remove unnecessary eval when its not a literal
                            let mut istate_new = InterpreterState {
                                stack: Vec::new(),
                                vars: self.vars.to_owned(),
                                globals: self.globals.clone(),
                                delims: Vec::new(),
                                ext_fns: self.ext_fns
                            };
                            if let Value::Array(a) = array {
                                if let Value::Ident(ref i) = val_name {
                                    if let Value::Block(ref b) = block {
                                        istate_new.add_var(i);
                                        for val in a {
                                            istate_new.set_var(i, val);
                                            istate_new.run(b);
                                        }
                                        for var in self.vars.iter_mut() {
                                            *var.1 = istate_new.get_var(var.0).unwrap().clone();
                                        }
                                    } else {
                                        println!("{:?}", self);
                                        panic!("not a block {:?}", block);
                                    }
                                } else {
                                    println!("{:?}", self);
                                    panic!("not an ident {:?}", val_name);
                                }
                            } else {
                                println!("{:?}", self);
                                panic!("not an array {:?}", array);
                            }
                            self.globals = istate_new.globals;
                        }
                        Keyword::If => {
                            let block = self.get_value().unwrap();
                            let cond = self.get_int().unwrap();
                            if cond != 0 {
                                if let Value::Block(ref b) = block {
                                    let mut istate_new = InterpreterState {
                                        stack: Vec::new(),
                                        vars: self.vars.to_owned(),
                                        globals: self.globals.to_owned(),
                                        delims: Vec::new(),
                                        ext_fns: self.ext_fns
                                    };
                                    istate_new.run(b);
                                    for var in self.vars.iter_mut() {
                                        *var.1 = istate_new.get_var(var.0).unwrap().clone();
                                    }
                                    self.globals = istate_new.globals;
                                } else {
                                    println!("{:?}", self);
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

}

pub fn tokenize(fortnite: &str) -> Vec<Value> {
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
                        "global" => {
                            vals.push(Value::Keyword(Keyword::Global));
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
    vals
}
