use std::collections::HashMap;
use tera::{Tera, Value};

pub(super) fn extend(mut tera: Tera) -> Tera {
    let map_str_with = |map_fn: fn(&str) -> String| {
        move |v: &Value, _: &HashMap<String, Value>| match v.as_str() {
            Some(s) => Ok(Value::String(map_fn(s))),
            None => Err("Expected a string".into()),
        }
    };

    tera.register_filter(
        "camel_case",
        map_str_with(heck::ToLowerCamelCase::to_lower_camel_case),
    );
    tera.register_filter("kebab_case", map_str_with(heck::ToKebabCase::to_kebab_case));
    tera.register_filter(
        "pascal_case",
        map_str_with(heck::ToPascalCase::to_pascal_case),
    );
    tera.register_filter("snake_case", map_str_with(heck::ToSnakeCase::to_snake_case));
    tera.register_filter("title_case", map_str_with(heck::ToTitleCase::to_title_case));
    tera.register_filter("train_case", map_str_with(heck::ToTrainCase::to_train_case));
    tera.register_filter(
        "upper_kebab_case",
        map_str_with(heck::ToShoutyKebabCase::to_shouty_kebab_case),
    );
    tera.register_filter(
        "upper_snake_case",
        map_str_with(heck::ToShoutySnakeCase::to_shouty_snake_case),
    );

    tera
}
