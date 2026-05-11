use godot::prelude::*;
use snakerobots_shared::{
    lang::error::context::{ErrorContext as LangErrorContext, SpanType},
    logic::robot::lang::LangRobotError
};

pub fn convert_error(code: &str, err: Box<dyn std::error::Error>) -> Array<Gd<SrLangError>> {
    if let Some(lang_err) = err.downcast_ref::<LangRobotError>() {
        return convert_lang_error(code, lang_err);
    }
    create_error(&err.to_string())
}

pub fn create_error(message: &str) -> Array<Gd<SrLangError>> {
    vec![SrLangError::create(message)].into_iter().collect()
}

pub fn convert_lang_error(code: &str, err: &LangRobotError) -> Array<Gd<SrLangError>> {
    match err {
        LangRobotError::Compile(errs) => {
            convert_error_context_multiple(code, errs)
        },
        LangRobotError::Runtime(ctx) => {
            vec![convert_error_context(code, ctx)].into_iter().collect()
        },
        err => {
            vec![SrLangError::create(&err.to_string())].into_iter().collect()
        }
    }
}

pub fn convert_error_context_multiple<T>(code: &str, errs: &Vec<LangErrorContext<T>>) -> Array<Gd<SrLangError>>
where 
    T: std::fmt::Display,
{
    errs.into_iter()
        .map(|ctx| convert_error_context(code, ctx))
        .collect()
}

pub fn convert_error_context<T>(code: &str, ctx: &LangErrorContext<T>) -> Gd<SrLangError>
where 
    T: std::fmt::Display,
{
    Gd::from_object(SrLangError {
        span: SrLangErrorSpan::from_context(ctx, code).map(|x| Gd::from_object(x)),
        message: ctx.error.to_string().to_godot(),
    })
}

#[derive(GodotClass, Default)]
#[class(init, base=RefCounted)]
pub struct SrLangError {
    #[var]
    pub span: Option<Gd<SrLangErrorSpan>>,
    #[var]
    pub message: GString,
}

impl SrLangError {
    pub fn create(message: &str) -> Gd<SrLangError> {
        Gd::from_object(Self {
            span: None,
            message: message.to_godot(),
        })
    }
}

#[derive(GodotClass, Default)]
#[class(init, base=RefCounted)]
pub struct SrLangErrorSpan {
    #[var]
    pub line_from: i64,
    #[var]
    pub line_to: i64,
    #[var]
    pub column_from: i64,
    #[var]
    pub column_to: i64,
    #[var]
    pub char_from: i64,
    #[var]
    pub char_to: i64,
}

impl SrLangErrorSpan {
    pub fn from_context<T>(ctx: &LangErrorContext<T>, code: &str) -> Option<Self> {
        match ctx.span_type {
            SpanType::SEGMENT(span) => {
                let (line_from, column_from) = get_line_and_column(code, span.from)?;
                let (line_to, column_to) = get_line_and_column(code, span.to)?;

                Some(Self {
                    line_from,
                    line_to,
                    column_from,
                    column_to,
                    char_from: span.from as i64,
                    char_to: span.to as i64,
                })
            },
            SpanType::LINE(l) => {
                let mut i = 0;
                let mut c = 0;
                let mut len = 0;

                for line in code.split_inclusive('\n') {
                    c += len;
                    len = line.chars().count();

                    if i == l {
                        break
                    }

                    i += 1;
                }

                // check if outside the bounds of the code
                if i < l {
                    None
                } else {
                    Some(Self {
                        line_from: l as i64,
                        line_to: l as i64,
                        column_from: 0,
                        column_to: len as i64,
                        char_from: c as i64,
                        char_to: (c + len) as i64,
                    })
                }
            },
            SpanType::UNKNOWN => None
        }
    }
}

fn get_line_and_column(s: &str, i: usize) -> Option<(i64, i64)> {
    let mut j = 0usize;
    let mut line = 0;
    let mut col = 0;

    for char in s.chars() {
        if j == i {
            break;
        }

        j += 1;
        col += 1;
        if char == '\n' {
            line += 1;
            col = 0;
        }
    }

    // check if outside the bounds of the code
    if j < i {
        None
    } else {
        Some((line, col))
    }
}

