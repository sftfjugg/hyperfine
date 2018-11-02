/// This module contains common units.

/// Type alias for unit of time
pub type Second = f64;

/// Supported time units
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Second,
    MilliSecond,
}

impl Unit {
    /// The abbreviation of the Unit.
    pub fn short_name(&self) -> String {
        match *self {
            Unit::Second => String::from("s"),
            Unit::MilliSecond => String::from("ms"),
        }
    }

    /// Returns the Second value formatted for the Unit.
    pub fn format(&self, value: Second) -> String {
        match *self {
            Unit::Second => format!("{:.3}", value),
            Unit::MilliSecond => format!("{:.1}", value * 1e3),
        }
    }
}

#[test]
fn test_unit_short_name() {
    assert_eq!("s", Unit::Second.short_name());
    assert_eq!("ms", Unit::MilliSecond.short_name());
}

// Note - the values are rounded when formatted.
#[test]
fn test_unit_format() {
    let value: Second = 123.456789;
    assert_eq!("123.457", Unit::Second.format(value));
    assert_eq!("123456.8", Unit::MilliSecond.format(value));
}
