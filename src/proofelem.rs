use core::str::FromStr;
use std::string::ToString;

pub enum Elem<T> {
    Left(T),
    Right(T),
}

#[derive(Debug)]
pub enum ElemFromStrErr<T: FromStr> {
    BadPrefix,
    Inner(T::Err),
}

impl<T: FromStr> FromStr for Elem<T> {
    type Err = ElemFromStrErr<T>;
    fn from_str(a: &str) -> Result<Self, ElemFromStrErr<T>> {
        if a.starts_with("l") {
            T::from_str(&a[1..])
                .map(Self::Left)
                .map_err(ElemFromStrErr::Inner)
        } else if a.starts_with("r") {
            T::from_str(&a[1..])
                .map(Self::Right)
                .map_err(ElemFromStrErr::Inner)
        } else {
            Err(ElemFromStrErr::BadPrefix)
        }
    }
}

impl<T: ToString> ToString for Elem<T> {
    fn to_string(&self) -> String {
        let (c, a) = match self {
            Self::Left(a) => ('l', a),
            Self::Right(a) => ('r', a),
        };
        format!("{}{}", c, a.to_string())
    }
}

impl<T> ToString for ElemFromStrErr<T>
where
    T: FromStr,
    T::Err: ToString,
{
    fn to_string(&self) -> String {
        match self {
            Self::BadPrefix => "string must be prefixed with either 'l' or 'r'".to_string(),
            Self::Inner(a) => a.to_string(),
        }
    }
}
