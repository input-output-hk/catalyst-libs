//! A cddl type defition in which all pest parsed AST will be transformed during the
//! preprocessing step.

pub(crate) type CddlTypeName = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CddlType {
    TypeName(CddlTypeName),
    Choice(Vec<CddlType>),
    CborType(CborType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
