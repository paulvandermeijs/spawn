use anyhow::Result;
use log::info;
use serde::{
    Serialize,
    ser::{SerializeMap, SerializeSeq},
};
use std::cell::RefCell;
use std::path::Path;
use steel::{SteelErr, SteelVal, steel_vm::engine::Engine};
use tera::Context;

const FUNCTION_CWD: &str = "cwd";
const FUNCTION_INFO: &str = "info";
const FUNCTION_CONTEXT: &str = "context";
const FUNCTION_MESSAGE: &str = "message";
const FUNCTION_HELP_MESSAGE: &str = "help-message ";
const FUNCTION_PLACEHOLDER: &str = "placeholder ";
const FUNCTION_INITIAL_VALUE: &str = "initial-value ";
const FUNCTION_DEFAULT: &str = "default ";
const FUNCTION_VALIDATE: &str = "validate";
const FUNCTION_OPTIONS: &str = "options";

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

        let plugins_data = std::fs::read_to_string(path)?;

        plugins_data.try_into()
    }

    fn call_function(
        &self,
        name: &str,
        arguments: Vec<SteelVal>,
    ) -> Option<Result<SteelVal, SteelErr>> {
        let mut vm = self.vm.borrow_mut();
        let vm = vm.as_mut()?;

        if !vm.global_exists(name) {
            return None;
        }

        let result = vm.call_function_by_name_with_args(name, arguments);

        Some(result)
    }

    fn map_string_with_function(
        &self,
        name: &str,
        arguments: &[&str],
        default: Option<&str>,
    ) -> Result<String> {
        let default = default.or_else(|| arguments.first().copied()).unwrap_or("");
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

    pub(crate) fn cwd(&self, cwd: &str) -> Result<String> {
        self.map_string_with_function(FUNCTION_CWD, &[cwd], None)
    }

    pub(crate) fn info(&self, info: Option<&str>) -> Result<Option<String>> {
        let info = info.unwrap_or("");
        let info = self.map_string_with_function(FUNCTION_INFO, &[info], None)?;
        let info = if info.is_empty() { None } else { Some(info) };

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
        self.map_string_with_function(FUNCTION_MESSAGE, &[identifier, message], Some(message))
    }

    pub(crate) fn help_message(
        &self,
        identifier: &str,
        help_message: Option<&str>,
    ) -> Result<Option<String>> {
        let help_message = help_message.unwrap_or("");
        let help_message = self.map_string_with_function(
            FUNCTION_HELP_MESSAGE,
            &[identifier, help_message],
            Some(help_message),
        )?;
        let help_message = if help_message.is_empty() {
            None
        } else {
            Some(help_message)
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
            &[identifier, placeholder],
            Some(placeholder),
        )?;
        let placeholder = if placeholder.is_empty() {
            None
        } else {
            Some(placeholder)
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
            &[identifier, initial_value],
            Some(initial_value),
        )?;
        let initial_value = if initial_value.is_empty() {
            None
        } else {
            Some(initial_value)
        };

        Ok(initial_value)
    }

    pub(crate) fn default(
        &self,
        identifier: &str,
        default: Option<&str>,
    ) -> Result<Option<String>> {
        let default = default.unwrap_or("");
        let default =
            self.map_string_with_function(FUNCTION_DEFAULT, &[identifier, default], Some(default))?;
        let default = if default.is_empty() {
            None
        } else {
            Some(default)
        };

        Ok(default)
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

    pub(crate) fn options(&self, identifier: &str, value: &[String]) -> Result<Vec<String>> {
        let value_argument = value
            .iter()
            .map(|v| SteelVal::StringV(v.into()))
            .collect::<Vec<SteelVal>>();
        let value_argument = SteelVal::ListV(value_argument.into());
        let arguments = vec![identifier.to_string().into(), value_argument];
        let Some(result) = self.call_function(FUNCTION_OPTIONS, arguments) else {
            return Ok(value.to_vec());
        };
        let SteelVal::ListV(result) = result? else {
            return Err(anyhow::Error::msg(format!(
                "Plugin {FUNCTION_OPTIONS:?} should return a list"
            )));
        };
        let result = result.into_iter().try_fold(Vec::new(), |mut result, v| {
            let v = match v {
                SteelVal::StringV(steel_string) => steel_string.to_string(),
                SteelVal::NumV(int) => int.to_string(),
                SteelVal::IntV(int) => int.to_string(),
                SteelVal::CharV(char) => char.to_string(),
                _ => {
                    return Err(anyhow::Error::msg(
                        "List returned by {FUNCTION_OPTIONS:?} should only contain string values",
                    ));
                }
            };

            result.push(v);

            Ok(result)
        })?;

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
        match self.steel_val {
            SteelVal::BoolV(bool) => serializer.serialize_bool(*bool),
            SteelVal::NumV(num) => serializer.serialize_f64(*num),
            SteelVal::IntV(int) => serializer.serialize_i64(*int as i64),
            SteelVal::CharV(char) => serializer.serialize_char(*char),
            SteelVal::StringV(steel_string) | SteelVal::SymbolV(steel_string) => {
                serializer.serialize_str(&steel_string.to_string())
            }
            SteelVal::VectorV(steel_vector) => {
                let items: Vec<_> = steel_vector.iter().collect();
                let mut seq = serializer.serialize_seq(Some(items.len()))?;
                for element in &items {
                    let element: SerializableSteelVal = (*element).into();
                    seq.serialize_element(&element)?;
                }
                seq.end()
            }
            SteelVal::HashMapV(steel_hash_map) => {
                let entries: Vec<_> = steel_hash_map.iter().collect();
                let mut map = serializer.serialize_map(Some(entries.len()))?;
                for (key, value) in &entries {
                    let key: SerializableSteelVal = (*key).into();
                    let value: SerializableSteelVal = (*value).into();
                    map.serialize_entry(&key, &value)?;
                }
                map.end()
            }
            SteelVal::ListV(generic_list) => {
                let mut seq = serializer.serialize_seq(Some(generic_list.len()))?;
                for value in generic_list {
                    let value: SerializableSteelVal = value.into();
                    seq.serialize_element(&value)?;
                }
                seq.end()
            }
            _ => Err(serde::ser::Error::custom(format!(
                "unsupported SteelVal variant for serialization: {}",
                self.steel_val
            ))),
        }
    }
}

impl From<SerializableSteelVal<'_>> for String {
    fn from(value: SerializableSteelVal) -> Self {
        match value.steel_val {
            SteelVal::StringV(steel_string) => steel_string.to_string(),
            _ => value.steel_val.to_string(),
        }
    }
}
