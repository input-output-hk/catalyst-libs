//! A cddl type defition in which all pest parsed AST will be transformed during the
//! preprocessing step.

use std::collections::HashMap;

pub(crate) type CddlTypeName = String;

pub(crate) enum CddlType {
    TypeName(CddlTypeName),
    Choice(Vec<CddlType>),
    CborType(CborType),
}

pub(crate) enum CborType {
    Any,
    MajorType0,
    MajorType1,
    MajorType2,
    MajorType3,
    MajorType4,
    MajorType5,
    MajorType6(u64, Box<CddlType>),
    MajorType7(u64),
}

/// Stard prelude of cddl types
/// <https://datatracker.ietf.org/doc/html/rfc8610#appendix-D/>
#[allow(clippy::too_many_lines)]
pub(crate) fn standart_prelude() -> HashMap<CddlTypeName, CddlType> {
    vec![
        ("any".to_string(), CddlType::CborType(CborType::Any)),
        ("uint".to_string(), CddlType::CborType(CborType::MajorType0)),
        ("nint".to_string(), CddlType::CborType(CborType::MajorType1)),
        (
            "int".to_string(),
            CddlType::Choice(vec![
                CddlType::TypeName("uint".to_string()),
                CddlType::TypeName("nint".to_string()),
            ]),
        ),
        ("bstr".to_string(), CddlType::CborType(CborType::MajorType2)),
        ("bytes".to_string(), CddlType::TypeName("bstr".to_string())),
        ("tstr".to_string(), CddlType::CborType(CborType::MajorType3)),
        ("text".to_string(), CddlType::TypeName("tstr".to_string())),
        (
            "tdate".to_string(),
            CddlType::CborType(CborType::MajorType6(
                0,
                CddlType::TypeName("tstr".to_string()).into(),
            )),
        ),
        (
            "time".to_string(),
            CddlType::CborType(CborType::MajorType6(
                1,
                CddlType::TypeName("number".to_string()).into(),
            )),
        ),
        (
            "number".to_string(),
            CddlType::Choice(vec![
                CddlType::TypeName("int".to_string()),
                CddlType::TypeName("float".to_string()),
            ]),
        ),
        (
            "biguint".to_string(),
            CddlType::CborType(CborType::MajorType6(
                2,
                CddlType::TypeName("bstr".to_string()).into(),
            )),
        ),
        (
            "bignint".to_string(),
            CddlType::CborType(CborType::MajorType6(
                3,
                CddlType::TypeName("bstr".to_string()).into(),
            )),
        ),
        (
            "bigint".to_string(),
            CddlType::Choice(vec![
                CddlType::TypeName("biguint".to_string()),
                CddlType::TypeName("bignint".to_string()),
            ]),
        ),
        (
            "integer".to_string(),
            CddlType::Choice(vec![
                CddlType::TypeName("int".to_string()),
                CddlType::TypeName("bigint".to_string()),
            ]),
        ),
        (
            "unsigned".to_string(),
            CddlType::Choice(vec![
                CddlType::TypeName("uint".to_string()),
                CddlType::TypeName("biguint".to_string()),
            ]),
        ),
        (
            "float16".to_string(),
            CddlType::CborType(CborType::MajorType7(25)),
        ),
        (
            "float32".to_string(),
            CddlType::CborType(CborType::MajorType7(26)),
        ),
        (
            "float64".to_string(),
            CddlType::CborType(CborType::MajorType7(27)),
        ),
        (
            "false".to_string(),
            CddlType::CborType(CborType::MajorType7(20)),
        ),
        (
            "true".to_string(),
            CddlType::CborType(CborType::MajorType7(21)),
        ),
        (
            "nil".to_string(),
            CddlType::CborType(CborType::MajorType7(22)),
        ),
        (
            "undefined".to_string(),
            CddlType::CborType(CborType::MajorType7(23)),
        ),
    ]
    .into_iter()
    .collect()
}
