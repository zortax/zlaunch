//! Macro-based dispatch for ListItem enum.
//!
//! This macro eliminates repetitive match arm boilerplate when delegating
//! trait methods from the ListItem enum to its variants.

/// Dispatches a method call to the inner item of a ListItem variant.
///
/// Usage:
/// ```ignore
/// dispatch_item!(self, method_name)
/// dispatch_item!(self, method_name, arg1, arg2)
/// ```
macro_rules! dispatch_item {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            Self::Application(item) => item.$method($($arg),*),
            Self::Window(item) => item.$method($($arg),*),
            Self::Action(item) => item.$method($($arg),*),
            Self::Submenu(item) => item.$method($($arg),*),
            Self::Calculator(item) => item.$method($($arg),*),
            Self::Search(item) => item.$method($($arg),*),
            Self::Ai(item) => item.$method($($arg),*),
            Self::Theme(item) => item.$method($($arg),*),
        }
    };
}

pub(crate) use dispatch_item;
