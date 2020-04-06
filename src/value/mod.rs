use std::cmp::Ordering;
use std::fmt::{self, Display, Write};
use std::iter::Iterator;

use crate::color::Color;
use crate::common::{Brackets, ListSeparator, Op, QuoteKind};
use crate::error::SassResult;
use crate::unit::{Unit, UNIT_CONVERSION_TABLE};

pub(crate) use map::SassMap;
pub(crate) use number::Number;
pub(crate) use sass_function::SassFunction;

mod css_function;
mod map;
mod number;
mod ops;
mod parse;
mod sass_function;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Value {
    Important,
    True,
    False,
    Null,
    Dimension(Number, Unit),
    List(Vec<Value>, ListSeparator, Brackets),
    Color(Color),
    UnaryOp(Op, Box<Value>),
    BinaryOp(Box<Value>, Op, Box<Value>),
    Paren(Box<Value>),
    Ident(String, QuoteKind),
    Map(SassMap),
    ArgList(Vec<Value>),
    /// Returned by `get-function()`
    Function(SassFunction),
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Important => write!(f, "!important"),
            Self::Dimension(num, unit) => match unit {
                Unit::Mul(..) => {
                    eprintln!("Error: {}{} isn't a valid CSS value.", num, unit);
                    std::process::exit(1);
                }
                _ => write!(f, "{}{}", num, unit),
            },
            Self::Map(map) => write!(
                f,
                "({})",
                map.iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Function(func) => write!(f, "get-function(\"{}\")", func.name()),
            Self::List(vals, sep, brackets) => match brackets {
                Brackets::None => write!(
                    f,
                    "{}",
                    vals.iter()
                        .filter(|x| !x.is_null())
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(sep.as_str()),
                ),
                Brackets::Bracketed => write!(
                    f,
                    "[{}]",
                    vals.iter()
                        .filter(|x| !x.is_null())
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(sep.as_str()),
                ),
            },
            Self::Color(c) => write!(f, "{}", c),
            Self::UnaryOp(..) | Self::BinaryOp(..) => write!(
                f,
                "{}",
                match self.clone().eval() {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                }
            ),
            Self::Paren(val) => write!(f, "{}", val),
            Self::Ident(val, kind) => {
                if kind == &QuoteKind::None {
                    return write!(f, "{}", val);
                }
                let has_single_quotes = val.contains(|x| x == '\'');
                let has_double_quotes = val.contains(|x| x == '"');
                if has_single_quotes && !has_double_quotes {
                    write!(f, "\"{}\"", val)
                } else if !has_single_quotes && has_double_quotes {
                    write!(f, "'{}'", val)
                } else if !has_single_quotes && !has_double_quotes {
                    write!(f, "\"{}\"", val)
                } else {
                    let quote_char = match kind {
                        QuoteKind::Double => '"',
                        QuoteKind::Single => '\'',
                        _ => unreachable!(),
                    };
                    f.write_char(quote_char)?;
                    for c in val.chars() {
                        match c {
                            '"' | '\'' if c == quote_char => {
                                f.write_char('\\')?;
                                f.write_char(quote_char)?;
                            }
                            v => f.write_char(v)?,
                        }
                    }
                    f.write_char(quote_char)?;
                    Ok(())
                }
            }
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Null => write!(f, "null"),
            Self::ArgList(args) => write!(
                f,
                "{}",
                args.iter()
                    .filter(|x| !x.is_null())
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", "),
            ),
        }
    }
}

impl Value {
    pub fn is_null(&self) -> bool {
        match self {
            &Value::Null => true,
            Value::Ident(i, QuoteKind::None) if i.is_empty() => true,
            _ => false,
        }
    }

    pub fn is_true(&self) -> SassResult<bool> {
        match self {
            Value::Null | Value::False => Ok(false),
            Self::BinaryOp(..) | Self::Paren(..) | Self::UnaryOp(..) => {
                self.clone().eval()?.is_true()
            }
            _ => Ok(true),
        }
    }

    pub fn unquote(self) -> Self {
        match self {
            Self::Ident(s1, _) => Self::Ident(s1, QuoteKind::None),
            v => v,
        }
    }

    pub fn kind(&self) -> SassResult<&'static str> {
        match self {
            Self::Color(..) => Ok("color"),
            Self::Ident(..) | Self::Important => Ok("string"),
            Self::Dimension(..) => Ok("number"),
            Self::List(..) => Ok("list"),
            Self::Function(..) => Ok("function"),
            Self::ArgList(..) => Ok("arglist"),
            Self::True | Self::False => Ok("bool"),
            Self::Null => Ok("null"),
            Self::Map(..) => Ok("map"),
            Self::BinaryOp(..) | Self::Paren(..) | Self::UnaryOp(..) => self.clone().eval()?.kind(),
        }
    }

    pub fn bool(b: bool) -> Self {
        if b {
            Value::True
        } else {
            Value::False
        }
    }

    pub fn inspect(&self) -> String {
        match self {
            Value::List(v, _, brackets) if v.is_empty() => match brackets {
                Brackets::None => "()".to_string(),
                Brackets::Bracketed => "[]".to_string(),
            },
            Value::Function(f) => format!("get-function(\"{}\")", f.name()),
            v => v.to_string(),
        }
    }

    pub fn equals(self, other: Value) -> SassResult<bool> {
        Ok(match self.eval()? {
            Self::Ident(s1, ..) => match other {
                Self::Ident(s2, ..) => s1 == s2,
                _ => false,
            },
            Self::Dimension(n, unit) => match other {
                Self::Dimension(n2, unit2) => {
                    if !unit.comparable(&unit2) {
                        false
                    } else if unit == unit2 {
                        n == n2
                    } else if unit == Unit::None || unit2 == Unit::None {
                        false
                    } else {
                        n == (n2
                            * UNIT_CONVERSION_TABLE[&unit.to_string()][&unit2.to_string()].clone())
                    }
                }
                _ => false,
            },
            s => s == other.eval()?,
        })
    }

    pub fn unary_op_plus(self) -> SassResult<Self> {
        Ok(match self.eval()? {
            v @ Value::Dimension(..) => v,
            v => Value::Ident(format!("+{}", v), QuoteKind::None),
        })
    }

    pub fn eval(self) -> SassResult<Self> {
        match self {
            Self::BinaryOp(lhs, op, rhs) => match op {
                Op::Plus => *lhs + *rhs,
                Op::Minus => *lhs - *rhs,
                Op::Equal => Ok(Self::bool(lhs.equals(*rhs)?)),
                Op::NotEqual => Ok(Self::bool(!lhs.equals(*rhs)?)),
                Op::Mul => *lhs * *rhs,
                Op::Div => *lhs / *rhs,
                Op::Rem => *lhs % *rhs,
                Op::GreaterThan => lhs.cmp(*rhs, op),
                Op::GreaterThanEqual => lhs.cmp(*rhs, op),
                Op::LessThan => lhs.cmp(*rhs, op),
                Op::LessThanEqual => lhs.cmp(*rhs, op),
                Op::Not => unreachable!(),
                Op::And => Ok(if lhs.clone().is_true()? {
                    rhs.eval()?
                } else {
                    lhs.eval()?
                }),
                Op::Or => Ok(if lhs.is_true()? {
                    lhs.eval()?
                } else {
                    rhs.eval()?
                }),
            },
            Self::Paren(v) => v.eval(),
            Self::UnaryOp(op, val) => match op {
                Op::Plus => val.unary_op_plus(),
                Op::Minus => -*val,
                Op::Not => Ok(Self::bool(!val.eval()?.is_true()?)),
                _ => unreachable!(),
            },
            _ => Ok(self),
        }
    }

    pub fn cmp(self, mut other: Self, op: Op) -> SassResult<Value> {
        if let Self::Paren(..) = other {
            other = other.eval()?
        }
        let precedence = op.precedence();
        let ordering = match self {
            Self::Dimension(num, unit) => match &other {
                Self::Dimension(num2, unit2) => {
                    if !unit.comparable(&unit2) {
                        return Err(format!("Incompatible units {} and {}.", unit2, unit).into());
                    }
                    if &unit == unit2 {
                        num.cmp(num2)
                    } else if unit == Unit::None {
                        num.cmp(num2)
                    } else if unit2 == &Unit::None {
                        num.cmp(num2)
                    } else {
                        num.cmp(
                            &(num2.clone()
                                * UNIT_CONVERSION_TABLE[&unit.to_string()][&unit2.to_string()]
                                    .clone()),
                        )
                    }
                }
                Self::BinaryOp(..) => todo!(),
                v => return Err(format!("Undefined operation \"{} {} {}\".", v, op, other).into()),
            },
            Self::BinaryOp(left, op2, right) => {
                return if op2.precedence() >= precedence {
                    Self::BinaryOp(left, op2, right).eval()?.cmp(other, op)
                } else {
                    Self::BinaryOp(
                        left,
                        op2,
                        Box::new(Self::BinaryOp(right, op, Box::new(other)).eval()?),
                    )
                    .eval()
                }
            }
            Self::UnaryOp(..) | Self::Paren(..) => return self.eval()?.cmp(other, op),
            _ => return Err(format!("Undefined operation \"{} {} {}\".", self, op, other).into()),
        };
        match op {
            Op::GreaterThan => match ordering {
                Ordering::Greater => Ok(Self::True),
                Ordering::Less | Ordering::Equal => Ok(Self::False),
            },
            Op::GreaterThanEqual => match ordering {
                Ordering::Greater | Ordering::Equal => Ok(Self::True),
                Ordering::Less => Ok(Self::False),
            },
            Op::LessThan => match ordering {
                Ordering::Less => Ok(Self::True),
                Ordering::Greater | Ordering::Equal => Ok(Self::False),
            },
            Op::LessThanEqual => match ordering {
                Ordering::Less | Ordering::Equal => Ok(Self::True),
                Ordering::Greater => Ok(Self::False),
            },
            _ => unreachable!(),
        }
    }
}
