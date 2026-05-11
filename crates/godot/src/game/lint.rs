use godot::prelude::*;
use snakerobots_shared::lang;

use crate::game::error::SrLangError;

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct SrLangLinter {
}

#[godot_api]
impl SrLangLinter {
    #[func]
    pub fn lint(mut code: String, line: i64, column: i64) -> Gd<SrLangLintOutput> {
        code += snakerobots_shared::logic::robot::lang::LIB_CODE;

        let cursor = get_index(&code, line as usize, column as usize);

        let (map, errors) = lang::lint(code.clone(), cursor);

        let completions = map.into_iter()
            .map(|(value, completion_type)| SrLangLintCompletion::create(value, completion_type.as_string()))
            .collect::<Array<_>>();

        let errors = super::error::convert_error_context_multiple(&code, &errors);

        Gd::from_object(SrLangLintOutput { completions, errors })
    }
}

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct SrLangLintOutput {
    #[var]
    pub completions: Array<Gd<SrLangLintCompletion>>,
    #[var]
    pub errors: Array<Gd<SrLangError>>,
}

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct SrLangLintCompletion {
    #[var]
    pub value: GString,
    #[var]
    pub completion_type: GString,
}

#[godot_api]
impl SrLangLintCompletion {
    pub fn create(value: String, completion_type: &str) -> Gd<SrLangLintCompletion> {
        Gd::from_object(SrLangLintCompletion {
            value: value.to_godot(),
            completion_type: completion_type.to_godot(),
        })
    }
}

fn get_index(code: &str, line: usize, column: usize) -> usize {
    let mut i = 0usize;
    let mut c = 0usize;
    let mut len = 0usize;

    for line_str in code.split_inclusive('\n') {
        c += len;
        len = line_str.chars().count();

        if i == line {
            break
        }

        i += 1;
    }

    c + column
}
