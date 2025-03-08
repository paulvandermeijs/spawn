use anyhow::Result;
use log::info;
use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize,
};
use std::cell::RefCell;
use std::path::Path;
use steel::{steel_vm::engine::Engine, SteelErr, SteelVal};
use tera::Context;

const FUNCTION_CWD: &str = "cwd";
const FUNCTION_INFO: &str = "info";
const FUNCTION_CONTEXT: &str = "context";

pub struct Plugins {
    vm: RefCell<Option<Engine>>,
}

impl Plugins {
    pub(crate) fn try_from_file(path: &Path) -> Result<Self> {
        if !path.is_file() {
            let vm = RefCell::new(None);

            return Ok(Self { vm });
        }

        let plugins_data = std::fs::read_to_string(&path)?;
        let mut vm = steel::steel_vm::engine::Engine::new();

        info!("Using plugins file {path:?}");

        vm.run(plugins_data)?;

        let vm = RefCell::new(Some(vm));

        Ok(Self { vm })
    }

    fn call_function(
        &self,
        name: &str,
        arguments: Vec<SteelVal>,
    ) -> Option<Result<SteelVal, SteelErr>> {
        let mut vm = self.vm.borrow_mut();
        let Some(vm) = vm.as_mut() else {
            return None;
        };

        if !vm.global_exists(name) {
            return None;
        }

        let result = vm.call_function_by_name_with_args(name, arguments);

        Some(result)
    }

    pub(crate) fn cwd(&self, cwd: String) -> Result<String> {
        let arguments = vec![cwd.clone().into()];
        let Some(result) = self.call_function(FUNCTION_CWD, arguments) else {
            return Ok(cwd);
        };
        let SteelVal::StringV(cwd) = result? else {
            return Err(anyhow::Error::msg(format!(
                "Plugin {FUNCTION_CWD:?} should return a string"
            )));
        };
        let cwd = cwd.to_string();

        Ok(cwd)
    }

    pub(crate) fn info(&self, info: Option<String>) -> Result<Option<String>> {
        let arguments = vec![info.clone().unwrap_or("".to_string()).into()];
        let Some(result) = self.call_function(FUNCTION_INFO, arguments) else {
            return Ok(info);
        };
        let SteelVal::StringV(info) = result? else {
            return Err(anyhow::Error::msg(format!(
                "Plugin {FUNCTION_INFO:?} should return a string"
            )));
        };
        let info = info.to_string();
        let info = if !info.is_empty() { Some(info) } else { None };

        Ok(info)
    }

    pub(crate) fn context(&self, context: Context) -> Result<Context> {
        let arguments = vec![];
        let Some(result) = self.call_function(FUNCTION_CONTEXT, arguments) else {
            return Ok(context);
        };
        let SteelVal::HashMapV(context) = result? else {
            return Err(anyhow::Error::msg(format!(
                "Plugin {FUNCTION_CONTEXT:?} should return a hashmap"
            )));
        };
        let context = context
            .iter()
            .fold(Context::new(), |mut context, (key, val)| {
                let key: SerializableSteelVal = key.into();
                let val: SerializableSteelVal = val.into();

                context.insert(key, &val);

                context
            });

        Ok(context)
    }
}

struct SerializableSteelVal<'a> {
    steel_val: &'a SteelVal,
}

impl<'a> From<&'a SteelVal> for SerializableSteelVal<'a> {
    fn from(value: &'a SteelVal) -> Self {
        Self { steel_val: value }
    }
}

impl Serialize for SerializableSteelVal<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.steel_val.clone() {
            SteelVal::Closure(_gc) => todo!(),
            SteelVal::BoolV(bool) => serializer.serialize_bool(bool),
            SteelVal::NumV(int) => serializer.serialize_f64(int),
            SteelVal::IntV(int) => serializer.serialize_i128(int.try_into().unwrap()),
            SteelVal::Rational(_ratio) => todo!(),
            SteelVal::CharV(char) => serializer.serialize_char(char),
            SteelVal::VectorV(steel_vector) => {
                let steel_vector = steel_vector.iter();
                let mut seq = serializer.serialize_seq(Some(steel_vector.len()))?;
                for element in steel_vector {
                    let element: SerializableSteelVal = element.into();

                    seq.serialize_element(&element)?;
                }
                seq.end()
            }
            SteelVal::Void => todo!(),
            SteelVal::StringV(steel_string) => serializer.serialize_str(&steel_string.to_string()),
            SteelVal::FuncV(_) => todo!(),
            SteelVal::SymbolV(steel_string) => serializer.serialize_str(&steel_string.to_string()),
            SteelVal::Custom(_gc) => todo!(),
            SteelVal::HashMapV(steel_hash_map) => {
                let steel_hash_map = steel_hash_map.iter();
                let mut map = serializer.serialize_map(Some(steel_hash_map.len()))?;
                for (k, v) in steel_hash_map {
                    let k: SerializableSteelVal = k.into();
                    let v: SerializableSteelVal = v.into();

                    map.serialize_entry(&k, &v)?;
                }
                map.end()
            }
            SteelVal::HashSetV(_steel_hash_set) => todo!(),
            SteelVal::CustomStruct(_gc) => todo!(),
            SteelVal::PortV(_steel_port) => todo!(),
            SteelVal::IterV(_gc) => todo!(),
            SteelVal::ReducerV(_gc) => todo!(),
            SteelVal::FutureFunc(_) => todo!(),
            SteelVal::FutureV(_gc) => todo!(),
            SteelVal::StreamV(_gc) => todo!(),
            SteelVal::BoxedFunction(_gc) => todo!(),
            SteelVal::ContinuationFunction(_continuation) => todo!(),
            SteelVal::ListV(_generic_list) => todo!(),
            SteelVal::Pair(_gc) => todo!(),
            SteelVal::MutFunc(_) => todo!(),
            SteelVal::BuiltIn(_) => todo!(),
            SteelVal::MutableVector(_heap_ref) => todo!(),
            SteelVal::BoxedIterator(_gc) => todo!(),
            SteelVal::SyntaxObject(_gc) => todo!(),
            SteelVal::Boxed(_gc) => todo!(),
            SteelVal::HeapAllocated(_heap_ref) => todo!(),
            SteelVal::Reference(_gc) => todo!(),
            SteelVal::BigNum(_gc) => todo!(),
            SteelVal::BigRational(_gc) => todo!(),
            SteelVal::Complex(_gc) => todo!(),
            SteelVal::ByteVector(_steel_byte_vector) => todo!(),
        }
    }
}

impl From<SerializableSteelVal<'_>> for String {
    fn from(value: SerializableSteelVal) -> Self {
        match value.steel_val.clone() {
            SteelVal::StringV(steel_string) => steel_string.to_string(),
            _ => value.steel_val.to_string(),
        }
    }
}
