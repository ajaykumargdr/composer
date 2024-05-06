
pub const COMMON: &str = r#"#![allow(unused_imports)]
use paste::paste;
use super::*;
#[derive(Debug, Flow)]
pub struct WorkflowGraph {
    edges: Vec<(usize, usize)>,
    nodes: Vec<Box<dyn Execute>>,
}

impl WorkflowGraph {
    pub fn new(size: usize) -> Self {
        WorkflowGraph {
            nodes: Vec::with_capacity(size),
            edges: Vec::new(),
        }
    }
}

#[macro_export]
macro_rules! impl_execute_trait {
    ($ ($struct : ty), *) => {

            paste!{
                $( impl Execute for $struct {
                    fn execute(&mut self) -> Result<(),String>{
        self.run()
    }

    fn get_task_output(&self) -> Value {
        self.output().clone().into()
    }

    fn set_output_to_task(&mut self, input: Value) {
        self.setter(input)
    }
                }
            )*
        }
    };
}

#[allow(dead_code, unused)]
pub fn join_hashmap<T: PartialEq + std::hash::Hash + Eq + Clone, U: Clone, V: Clone>(
    first: HashMap<T, U>,
    second: HashMap<T, V>,
) -> HashMap<T, (U, V)> {
    let mut data: HashMap<T, (U, V)> = HashMap::new();
    for (key, value) in first {
        for (s_key, s_value) in &second {
            if key.clone() == *s_key {
                data.insert(key.clone(), (value.clone(), s_value.clone()));
            }
        }
    }
    data
}

#[no_mangle]
pub unsafe extern "C" fn free_memory(ptr: *mut u8, size: u32, alignment: u32) {
    let layout = Layout::from_size_align_unchecked(size as usize, alignment as usize);
    alloc::alloc::dealloc(ptr, layout);
}

#[link(wasm_import_module = "host")]
extern "C" {
    pub fn set_output(ptr: i32, size: i32);
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Output {
    pub result: Value,
}

#[no_mangle]
pub unsafe extern "C" fn memory_alloc(size: u32, alignment: u32) -> *mut u8 {
    let layout = Layout::from_size_align_unchecked(size as usize, alignment as usize);
    alloc::alloc::alloc(layout)
}

"#;

pub const LIB: &str =  r#"#![allow(unused_imports)]
#![allow(unused_macros)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(forgetting_copy_types)]
#![allow(unused_mut)]
#![allow(unused_must_use)]

mod common;
mod macros;
mod traits;
mod types;

use common::*;
use derive_enum_from_into::{EnumFrom, EnumTryInto};
use dyn_clone::{clone_trait_object, DynClone};
use macros::*;
use openwhisk_rust::*;
use paste::*;
use serde::{Deserialize, Serialize};
use serde_json::to_value;
use serde_json::Value;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Debug;
use traits::*;
use types::*;
use workflow_macro::Flow;
extern crate alloc;
use codec::{Decode, Encode};
use core::alloc::Layout;

#[no_mangle]
pub fn _start(ptr: *mut u8, length: i32) {
    let result: Value;
    unsafe {
        let mut vect = Vec::new();
        for i in 1..=length {
            if let Some(val_back) = ptr.as_ref() {
                vect.push(val_back.clone());
            }
            *ptr = *ptr.add(i as usize);
        }
        result = serde_json::from_slice(&vect).unwrap();
    }

    let res = main(result);
    let output = Output {
        result: serde_json::to_value(res).unwrap(),
    };
    let serialized = serde_json::to_vec(&output).unwrap();
    let size = serialized.len() as i32;
    let ptr = serialized.as_ptr();
    std::mem::forget(ptr);
    unsafe {
        set_output(ptr as i32, size);
    }
}

"#;
pub const TRAIT: &str =  r#"

use super::*;

pub trait Execute : Debug + DynClone  {
    fn execute(&mut self)-> Result<(),String>;
    fn get_task_output(&self)->Value;
    fn set_output_to_task(&mut self, inp: Value);
}

clone_trait_object!(Execute);

"#;
pub const MACROS: &str =  r#"
use super::*;

#[macro_export]
macro_rules! make_input_struct {
    (
        $x:ident,
        [$(
            $(#[$default_derive:stmt])?
            $visibility:vis $element:ident : $ty:ty),*],
        [$($der:ident),*]
) => {
        #[derive($($der),*)]
            pub struct $x { 
            $(
                $(#[serde(default=$default_derive)])?
                $visibility  $element: $ty
            ),*
        }
    }
}

#[macro_export]
macro_rules! make_main_struct {
    (
        $name:ident,
        $input:ty,
        [$($der:ident),*],
        [$($key:ident : $val:expr),*],
        $output_field: ident
) => {
        #[derive($($der),*)]
        $(
            #[$key = $val]
        )*
        pub struct $name {
            action_name: String,
            pub input: $input,
            pub output: Value,
            pub mapout: Value
        }
        impl $name{
            pub fn output(&self) -> Value {
                self.$output_field.clone()
            }
        }
    }
}

#[macro_export]
macro_rules! impl_new {
    (
        $name:ident,
        $input:ident,
        []
    ) => {
        impl $name{
            pub fn new(action_name:String) -> Self{
                Self{
                    action_name,
                    input: $input{
                        ..Default::default()
                    },
                    ..Default::default()
                }      
            }
        }
    };
    (
        $name:ident,
        $input:ident,
        [$($element:ident : $ty:ty),*]
    ) => {
        impl $name{
            pub fn new($( $element: $ty),*, action_name:String) -> Self{
                Self{
                    action_name,
                    input: $input{
                        $($element),*,
                        ..Default::default()
                    },
                    ..Default::default()
                }      
            }
        }
    }
}

#[macro_export]
macro_rules! impl_setter {
    (
        $name:ty,
        [$($element:ident : $key:expr),*]
    ) => {
        impl $name{
            pub fn setter(&mut self, value: Value) {
                $(
                    let val = value.get($key).unwrap();
                    self.input.$element = serde_json::from_value(val.clone()).unwrap();
                )*
            }
        }
    }
}

#[macro_export]
macro_rules! impl_map_setter {
    (
        $name:ty,
        $element:ident : $key:expr,  
        $typ_name : ty,
        $out:expr
    ) => {
        impl $name {
            pub fn setter(&mut self, val: Value) {
                
                    let value = val.get($key).unwrap();
                    let value = serde_json::from_value::<Vec<$typ_name>>(value.clone()).unwrap();
                    let mut map: HashMap<_, _> = value
                        .iter()
                        .map(|x| {
                            self.input.$element = x.to_owned() as $typ_name;
                            self.run();
                            (x.to_owned(), self.output.get($out).unwrap().to_owned())
                        })
                        .collect();
                    self.mapout = to_value(map).unwrap();
                
            }
        }
    }
    }

#[macro_export]
macro_rules! impl_concat_setter {
    (
        $name:ty,
        $input:ident
    ) => {
        impl $name{
            pub fn setter(&mut self, val: Value) {
                
                    let val: Vec<Value> = serde_json::from_value(val).unwrap();
                    let res = join_hashmap(
                        serde_json::from_value(val[0].to_owned()).unwrap(),
                        serde_json::from_value(val[1].to_owned()).unwrap(),
                    );
                    self.input.$input = res;
            }
        }
    }
}

#[allow(unused)]
#[macro_export]
macro_rules! impl_combine_setter {
    (
        $name:ty,
        [$(
            $(($value_input:ident))?
            $([$index:expr])?
            $element:ident : $key:expr),*]
    ) => {
        impl $name{
            pub fn setter(&mut self, value: Value) {

                let value: Vec<Value> = serde_json::from_value(value).unwrap();
                $(
                    if stringify!($($value_input)*).is_empty(){
                        let val = value[$($index)*].get($key).unwrap();
                        self.input.$element = serde_json::from_value(val.clone()).unwrap();
                    }else{
                        self.input.$element = serde_json::from_value(value[$($index)*].to_owned()).unwrap();
                    }
                )*
            }
        }
    }
}


"#;
pub const CARGO: &str = r#"

[package]
name = "boilerplate"
version = "0.0.1"
edition = "2018"


[lib]
crate-type = ["cdylib"]

[profile.release]
lto = true
codegen-units = 1
overflow-checks = true
# Tell `rustc` to optimize for small code size.
opt-level = "z"
debug = false

[workspace]

[dependencies]
derive-enum-from-into = "0.1.1"
serde_derive = "1.0.192"
paste = "1.0.7"
dyn-clone = "1.0.7"
workflow_macro = "0.0.3"
openwhisk-rust = "0.1.2"
serde_json = { version = "1.0", features = ["raw_value"] }
serde = { version = "1.0.192", features = ["derive"] }
codec = { package = "parity-scale-codec", features = [
    "derive",
], version = "3.1.5" }

"#;