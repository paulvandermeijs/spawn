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
const FUNCTION_MESSAGE: &str = "message";
const FUNCTION_HELP_MESSAGE: &str = "help-message ";
const FUNCTION_PLACEHOLDER: &str = "placeholder ";
const FUNCTION_INITIAL_VALUE: &str = "initial-value ";
const FUNCTION_DEFAULT: &str = "default ";
const FUNCTION_SUGGESTIONS: &str = "suggestions";
const FUNCTION_COMPLETION: &str = "completion";
const FUNCTION_FORMAT: &str = "format";
const FUNCTION_VALIDATE: &str = "validate";

#[derive(Clone)]
pub(crate) struct Plugins {
    vm: RefCell<Option<Engine>>,
}

impl Plugins {
    pub(super) fn try_from_file(path: &Path) -> Result<Self> {
        if !path.is_file() {
            let vm = RefCell::new(None);

            return Ok(Self { vm });
        }

        info!("Using plugins file {path:?}");

        let plugins_data = std::fs::read_to_string(&path)?;

        plugins_data.try_into()
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

    fn map_string_with_function(
        &self,
        name: &str,
        arguments: Vec<&str>,
        default: Option<&str>,
    ) -> Result<String> {
        let default = default.unwrap_or(arguments.iter().next().unwrap());
        let arguments = arguments
            .iter()
            .map(|a| a.to_string().into())
            .collect::<Vec<SteelVal>>();
        let Some(result) = self.call_function(name, arguments) else {
            return Ok(default.to_string());
        };
        let SteelVal::StringV(result) = result? else {
            return Err(anyhow::Error::msg(format!(
                "Plugin {name:?} should return a string"
            )));
        };
        let result = result.to_string();

        Ok(result)
    }

    pub(crate) fn cwd(&self, cwd: String) -> Result<String> {
        self.map_string_with_function(FUNCTION_CWD, vec![&cwd], None)
    }

    pub(crate) fn info(&self, info: Option<&str>) -> Result<Option<String>> {
        let info = info.unwrap_or("");
        let info = self.map_string_with_function(FUNCTION_INFO, vec![info], None)?;
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

    pub(crate) fn message(&self, identifier: &str, message: &str) -> Result<String> {
        self.map_string_with_function(FUNCTION_MESSAGE, vec![identifier, message], Some(message))
    }

    pub(crate) fn help_message(
        &self,
        identifier: &str,
        help_message: Option<&str>,
    ) -> Result<Option<String>> {
        let help_message = help_message.unwrap_or("");
        let help_message = self.map_string_with_function(
            FUNCTION_HELP_MESSAGE,
            vec![identifier, help_message],
            Some(help_message),
        )?;
        let help_message = if !help_message.is_empty() {
            Some(help_message)
        } else {
            None
        };

        Ok(help_message)
    }

    pub(crate) fn placeholder(
        &self,
        identifier: &str,
        placeholder: Option<&str>,
    ) -> Result<Option<String>> {
        let placeholder = placeholder.unwrap_or("");
        let placeholder = self.map_string_with_function(
            FUNCTION_PLACEHOLDER,
            vec![identifier, placeholder],
            Some(placeholder),
        )?;
        let placeholder = if !placeholder.is_empty() {
            Some(placeholder)
        } else {
            None
        };

        Ok(placeholder)
    }

    pub(crate) fn initial_value(
        &self,
        identifier: &str,
        initial_value: Option<&str>,
    ) -> Result<Option<String>> {
        let initial_value = initial_value.unwrap_or("");
        let initial_value = self.map_string_with_function(
            FUNCTION_INITIAL_VALUE,
            vec![identifier, initial_value],
            Some(initial_value),
        )?;
        let initial_value = if !initial_value.is_empty() {
            Some(initial_value)
        } else {
            None
        };

        Ok(initial_value)
    }

    pub(crate) fn default(
        &self,
        identifier: &str,
        default: Option<&str>,
    ) -> Result<Option<String>> {
        let default = default.unwrap_or("");
        let default = self.map_string_with_function(
            FUNCTION_DEFAULT,
            vec![identifier, default],
            Some(default),
        )?;
        let default = if !default.is_empty() {
            Some(default)
        } else {
            None
        };

        Ok(default)
    }

    pub(crate) fn suggestions(&self, identifier: &str, value: &str) -> Result<Vec<String>> {
        let arguments = vec![identifier.to_string().into(), value.to_string().into()];
        let Some(result) = self.call_function(FUNCTION_SUGGESTIONS, arguments) else {
            return Ok(Vec::new());
        };
        let SteelVal::ListV(result) = result? else {
            return Err(anyhow::Error::msg(format!(
                "Plugin {FUNCTION_SUGGESTIONS:?} should return a list"
            )));
        };
        let result = result.into_iter().try_fold(Vec::new(), |mut result, v| {
            let v = match v {
                SteelVal::StringV(steel_string) => steel_string.to_string(),
                SteelVal::NumV(int) => int.to_string(),
                SteelVal::IntV(int) => int.to_string(),
                SteelVal::CharV(char) => char.to_string(),
                _ => return Err(anyhow::Error::msg(
                    "List returned by {FUNCTION_SUGGESTIONS:?} should only contain string values",
                )),
            };

            result.push(v);

            Ok(result)
        })?;

        Ok(result)
    }

    pub(crate) fn completion(
        &self,
        identifier: &str,
        value: &str,
        selected_suggestion: Option<&str>,
    ) -> Result<Option<String>> {
        let suggestion = selected_suggestion.map(|v| v.to_string());
        let arguments = vec![
            identifier.to_string().into(),
            value.to_string().into(),
            suggestion.clone().into(),
        ];
        let Some(result) = self.call_function(FUNCTION_COMPLETION, arguments) else {
            return Ok(suggestion);
        };
        let SteelVal::StringV(result) = result? else {
            return Err(anyhow::Error::msg(format!(
                "Plugin {FUNCTION_COMPLETION:?} should return a string"
            )));
        };
        let result = result.to_string();
        let result = if !result.is_empty() {
            Some(result)
        } else {
            None
        };

        Ok(result)
    }

    pub(crate) fn format(&self, identifier: &str, value: &str) -> Result<String> {
        let arguments = vec![identifier.to_string().into(), value.to_string().into()];
        let Some(result) = self.call_function(FUNCTION_FORMAT, arguments) else {
            return Ok(value.to_string());
        };
        let SteelVal::StringV(result) = result? else {
            return Err(anyhow::Error::msg(format!(
                "Plugin {FUNCTION_FORMAT:?} should return a string"
            )));
        };
        let result = result.to_string();

        Ok(result)
    }

    pub(crate) fn validate(&self, identifier: &str, value: &str) -> Result<Result<(), String>> {
        let arguments = vec![identifier.to_string().into(), value.to_string().into()];
        let Some(result) = self.call_function(FUNCTION_VALIDATE, arguments) else {
            return Ok(Ok(()));
        };

        let result = match result? {
            SteelVal::BoolV(bool) => {
                if bool {
                    Ok(())
                } else {
                    Err("Invalid value".into())
                }
            }
            SteelVal::StringV(steel_string) => Err(steel_string.to_string()),
            _ => Ok(()),
        };

        Ok(result)
    }
}

impl TryFrom<String> for Plugins {
    type Error = anyhow::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let mut vm = steel::steel_vm::engine::Engine::new();

        vm.run(value)?;

        let vm = RefCell::new(Some(vm));

        Ok(Self { vm })
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
                for (key, value) in steel_hash_map {
                    let key: SerializableSteelVal = key.into();
                    let value: SerializableSteelVal = value.into();

                    map.serialize_entry(&key, &value)?;
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
            SteelVal::ListV(generic_list) => {
                let mut seq = serializer.serialize_seq(Some(generic_list.len()))?;

                for value in &generic_list {
                    let value: SerializableSteelVal = value.into();

                    seq.serialize_element(&value)?;
                }

                seq.end()
            }
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
