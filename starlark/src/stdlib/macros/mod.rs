// Copyright 2018 The Starlark in Rust Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Define the `starlark_module!` macro to reduce written boilerplate when adding
//! native functions to starlark.

pub mod param;

/// Generate param name for named or unnamed parameter
#[doc(hidden)]
#[macro_export]
macro_rules! starlark_param_name {
    ((named) $ident:tt) => {
        stringify!($ident)
    };
    (# $ident:tt) => {
        concat!("$", stringify!($ident))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! starlark_signature {
    ($signature:ident) => {};
    ($signature:ident call_stack $e:ident $(,$($rest:tt)+)?) => {
        $( starlark_signature!($signature $($rest)+) )?;
    };
    ($signature:ident env $e:ident $(,$($rest:tt)+)?) => {
        $( starlark_signature!($signature $($rest)+) )?;
    };
    ($signature:ident * $t:ident $(: $pt:ty)? $(,$($rest:tt)+)?) => {
        $signature.push($crate::values::function::FunctionParameter::ArgsArray(stringify!($t).to_owned()));
        $( starlark_signature!($signature $($rest)+) )?;
    };
    ($signature:ident ** $t:ident $(: $pt:ty)? $(,$($rest:tt)+)?) => {
        $signature.push($crate::values::function::FunctionParameter::KWArgsDict(stringify!($t).to_owned()));
        $( starlark_signature!($signature $($rest)+) )?;
    };

    // insert `(named)` tt if param is not unnamed
    ($signature:ident $t:ident $($rest:tt)*) => {
        starlark_signature!($signature (named) $t $($rest)*)
    };
    ($signature:ident ? $t:ident $($rest:tt)*) => {
        starlark_signature!($signature ? (named) $t $($rest)*)
    };

    // handle params without default value (both named and unnamed)
    ($signature:ident $is_named:tt $t:ident $(: $pt:ty)? $(,$($rest:tt)+)?) => {
        $signature.push($crate::values::function::FunctionParameter::Normal(starlark_param_name!($is_named $t).to_owned()));
        $( starlark_signature!($signature $($rest)+) )?;
    };
    ($signature:ident ? $is_named:tt $t:ident $(: $pt:ty)? $(,$($rest:tt)+)?) => {
        $signature.push($crate::values::function::FunctionParameter::Optional(starlark_param_name!($is_named $t).to_owned()));
        $( starlark_signature!($signature $($rest)+) )?;
    };

    // handle params with default value (both named and unnamed)
    ($signature:ident $is_named:tt $t:ident : $pt:ty = $e:expr $(,$($rest:tt)+)?) => {
        $signature.push(
            $crate::values::function::FunctionParameter::WithDefaultValue(
                starlark_param_name!($is_named $t).to_owned(),
                // explicitly specify parameter type to:
                // * verify that default value is convertible to required type
                // * help type inference find type parameters
                ::std::convert::From::<starlark_parse_param_type!(1 : $pt)>::from($e)
            )
        );
        $( starlark_signature!($signature $($rest)+) )?;
    };
    ($signature:ident $is_named:tt $t:ident = $e:expr $(,$($rest:tt)+)?) => {
        $signature.push(
            $crate::values::function::FunctionParameter::WithDefaultValue(
                starlark_param_name!($is_named $t).to_owned(),
                $crate::values::Value::from($e)
            )
        );
        $( starlark_signature!($signature $($rest)+) )?;
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! starlark_parse_param_type {
    (1 : $pt:ty) => { $pt };
    (? : $pt:ty) => { $pt };
    (* : $pt:ty) => { $pt };
    (** : $pt:ty) => { $pt };
    (1) => {
        $crate::values::Value
    };
    (?) => {
        ::std::option::Option<$crate::values::Value>
    };
    (*) => {
        ::std::vec::Vec<$crate::values::Value>
    };
    (**) => {
        ::linked_hash_map::LinkedHashMap<::std::string::String, $crate::values::Value>
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! starlark_signature_extraction {
    ($args:ident $call_stack:ident $env:ident) => {};
    ($args:ident $call_stack:ident $env:ident call_stack $e:ident $(,$($rest:tt)+)?) => {
        let $e = $call_stack;
        $( starlark_signature_extraction!($args $call_stack $env $($rest)+) )?;
    };
    ($args:ident $call_stack:ident $env:ident env $e:ident $(,$($rest:tt)+)?) => {
        let $e = $env;
        $( starlark_signature_extraction!($args $call_stack $env $($rest)+) )?;
    };
    ($args:ident $call_stack:ident $env:ident * $t:ident $(: $pt:ty)? $(,$($rest:tt)+)?) => {
        #[allow(unused_mut)]
        let mut $t: starlark_parse_param_type!(* $(: $pt)?) =
            $args.next_arg()?.into_args_array(stringify!($t))?;
        $( starlark_signature_extraction!($args $call_stack $env $($rest)+) )?;
    };
    ($args:ident $call_stack:ident $env:ident ** $t:ident $(: $pt:ty)? $(,$($rest:tt)+)?) => {
        #[allow(unused_mut)]
        let mut $t: starlark_parse_param_type!(** $(: $pt)?) =
            $args.next_arg()?.into_kw_args_dict(stringify!($t))?;
        $( starlark_signature_extraction!($args $call_stack $env $($rest)+) )?;
    };

    // insert `(named)` tt if param is not unnamed
    ($args:ident $call_stack:ident $env:ident $t:ident $($rest:tt)*) => {
        starlark_signature_extraction!($args $call_stack $env (named) $t $($rest)*);
    };
    ($args:ident $call_stack:ident $env:ident ? $t:ident $($rest:tt)*) => {
        starlark_signature_extraction!($args $call_stack $env ? (named) $t $($rest)*);
    };

    ($args:ident $call_stack:ident $env:ident ? $is_named:tt $t:ident $(: $pt:ty)? $(,$($rest:tt)+)?) => {
        #[allow(unused_mut)]
        let mut $t: starlark_parse_param_type!(? $(: $pt)?) =
            $args.next_arg()?.into_optional(starlark_param_name!(# $t))?;
        $( starlark_signature_extraction!($args $call_stack $env $($rest)+) )?;
    };
    ($args:ident $call_stack:ident $env:ident $is_named:tt $t:ident $(: $pt:ty)? $(= $e:expr)? $(,$($rest:tt)+)?) => {
        #[allow(unused_mut)]
        let mut $t: starlark_parse_param_type!(1 $(: $pt)?) =
            $args.next_arg()?.into_normal(starlark_param_name!($is_named $t))?;
        $( starlark_signature_extraction!($args $call_stack $env $($rest)+) )?;
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! starlark_fun {
    ($(#[$attr:meta])* $fn:ident ( $($signature:tt)* ) { $($content:tt)* } $($($rest:tt)+)?) => {
        $(#[$attr])*
        fn $fn(
            __call_stack: &$crate::eval::call_stack::CallStack,
            __env: $crate::environment::TypeValues,
            mut args: $crate::values::function::ParameterParser,
        ) -> $crate::values::ValueResult {
            starlark_signature_extraction!(args __call_stack __env $($signature)*);
            args.check_no_more_args()?;
            $($content)*
        }
        $(starlark_fun! {
            $($rest)+
        })?
    };
    ($(#[$attr:meta])* $ty:ident . $fn:ident ( $($signature:tt)* ) { $($content:tt)* }
            $($($rest:tt)+)?) => {
        $(#[$attr])*
        fn $fn(
            __call_stack: &$crate::eval::call_stack::CallStack,
            __env: $crate::environment::TypeValues,
            mut args: $crate::values::function::ParameterParser,
        ) -> $crate::values::ValueResult {
            starlark_signature_extraction!(args __call_stack __env $($signature)*);
            args.check_no_more_args()?;
            $($content)*
        }
        $(starlark_fun! {
            $($rest)+
        })?
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! starlark_signatures {
    ($env:expr, $(#[$attr:meta])* $name:ident ( $($signature:tt)* ) { $($content:tt)* }
            $($($rest:tt)+)?) => {
        {
            let name = stringify!($name).trim_matches('_');
            #[allow(unused_mut)]
            let mut signature = Vec::new();
            starlark_signature!(signature $($signature)*);
            $env.set(name, $crate::values::function::NativeFunction::new(name.to_owned(), $name, signature)).unwrap();
        }
        $(starlark_signatures!{ $env,
            $($rest)+
        })?
    };
    ($env:expr, $(#[$attr:meta])* $ty:ident . $name:ident ( $($signature:tt)* ) { $($content:tt)* }
            $($($rest:tt)+)?) => {
        {
            let name = stringify!($name).trim_matches('_');
            let mut signature = Vec::new();
            starlark_signature!(signature $($signature)*);
            $env.add_type_value(stringify!($ty), name,
                $crate::values::function::NativeFunction::new(name.to_owned(), $name, signature));
        }
        $(starlark_signatures!{ $env,
            $($rest)+
        })?
    }
}

/// Declare a starlark module that store one or several function
///
/// To declare a module with name `name`, the macro would be called:
///
/// ```rust,ignore
/// starlark_module!{ name =>
///    // Starlark function definition goes there
/// }
/// ```
///
/// For instance, the following example would declare two functions `str`, `my_fun` and `dbg` in a
/// module named `my_starlark_module`:
///
/// ```rust
/// # #[macro_use] extern crate starlark;
/// # use starlark::values::*;
/// # use starlark::values::none::NoneType;
/// # use starlark::environment::Environment;
/// starlark_module!{ my_starlark_module =>
///     // Declare a 'str' function (_ are trimmed away and just here to avoid collision with
///     // reserved keyword)
///     // #a argument will be binded to a `a` Rust value, the '#' prevent the argument from
///     // being used by name when calling the method.
///     __str__(#a) {
///       Ok(Value::new(a.to_str().to_owned()))
///     }
///
///     // Declare a function my_fun that takes one positional parameter 'a', a named and
///     // positional parameter 'b', a args array 'args' and a keyword dictionary `kwargs`
///     my_fun(#a, b, c = 1, *args, **kwargs) {
///       // ...
/// # Ok(Value::new(true))
///     }
///
///     // Functions can optionally specify parameter types after colon.
///     // Parameter can be any type which implements `TryParamConvertFromValue`.
///     // When parameter type is not specified, it is defaulted to `Value`
///     // for regular parameters, `Vec<Value>` for `*args`
///     // and `LinkedHashMap<String, Value>` for `**kwargs`.
///     sqr(x: i64) {
///         Ok(Value::new(x * x))
///     }
///
///     // It is also possible to capture the call stack with
///     // `call_stack name` (type `Vec<String>`). For example a `dbg` function that print the
///     // the call stack:
///     dbg(call_stack cs) {
///        println!(
///            "In:{}",
///            cs.print_with_newline_before()
///        );
///        Ok(Value::from(NoneType::None))
///     }
/// }
/// #
/// # fn main() {
/// #    let env = my_starlark_module(Environment::new("test"));
/// #    assert_eq!(env.get("str").unwrap().get_type(), "function");
/// #    assert_eq!(env.get("my_fun").unwrap().get_type(), "function");
/// #    assert_eq!(env.get("sqr").unwrap().get_type(), "function");
/// # }
/// ```
///
/// The module would declare a function `my_starlark_module` that can be called to add the
/// corresponding functions to an environment.
///
/// ```
/// # #[macro_use] extern crate starlark;
/// # use starlark::values::*;
/// # use starlark::environment::Environment;
/// # starlark_module!{ my_starlark_module =>
/// #     __str__(#a) { Ok(Value::new(a.to_str().to_owned())) }
/// #     my_fun(#a, b, c = 1, *args, **kwargs) { Ok(Value::new(true)) }
/// # }
/// # fn main() {
/// #    let env =
/// my_starlark_module(Environment::new("test"))
/// # ;
/// #    assert_eq!(env.get("str").unwrap().get_type(), "function");
/// #    assert_eq!(env.get("my_fun").unwrap().get_type(), "function");
/// # }
/// ```
///
/// Additionally function might be declared for a type by prefixing them by `type.`, e.g the
/// definition of a `hello` function for the `string` type would look like:
///
/// ```rust
/// # #[macro_use] extern crate starlark;
/// # use starlark::values::*;
/// # use starlark::environment::{Environment, TypeValues};
/// starlark_module!{ my_starlark_module =>
///     // The first argument is always self in that module but we use "this" because "self" is a
///     // a rust keyword.
///     string.hello(this) {
///        Ok(Value::new(
///            format!("Hello, {}", this.to_str())
///        ))
///     }
/// }
/// #
/// # fn main() {
/// #    let env = TypeValues::new(my_starlark_module(Environment::new("test")));
/// #    assert_eq!(env.get_type_value(&Value::from(""), "hello").unwrap().get_type(), "function");
/// # }
/// ```
#[macro_export]
macro_rules! starlark_module {
    ($name:ident => $($t:tt)*) => (
        starlark_fun!{
            $($t)*
        }

        #[doc(hidden)]
        pub fn $name(env: $crate::environment::Environment) -> $crate::environment::Environment {
            starlark_signatures!{ env,
                $($t)*
            }
            env
        }
    )
}

/// Shortcut for returning an error from the code, message and label.
///
/// # Parameters:
///
/// * $code is a short code to uniquely identify the error.
/// * $message is the long explanation for the user of the error.
/// * $label is a a short description of the error to be put next to the code.
#[macro_export]
macro_rules! starlark_err {
    ($code:expr, $message:expr, $label:expr) => {
        return Err($crate::values::error::RuntimeError {
            code: $code,
            message: $message,
            label: $label,
        }
        .into());
    };
}

/// A shortcut to assert the type of a value
///
/// # Parameters:
///
/// * $e the value to check type for.
/// * $fn the function name (&'static str).
/// * $ty the expected type (ident)
#[macro_export]
macro_rules! check_type {
    ($e:ident, $fn:expr, $ty:ident) => {
        if $e.get_type() != stringify!($ty) {
            starlark_err!(
                INCORRECT_PARAMETER_TYPE_ERROR_CODE,
                format!(
                    concat!(
                        $fn,
                        "() expect a ",
                        stringify!($ty),
                        " as first parameter while got a value of type {}."
                    ),
                    $e.get_type()
                ),
                format!(
                    concat!("type {} while expected ", stringify!($ty)),
                    $e.get_type()
                )
            )
        }
    };
}

/// Convert 2 indices according to Starlark indices convertion for function like .index.
///
/// # Parameters:
///
/// * $this: the identifier of self object
/// * $start: the variable denoting the start index
/// * $end: the variable denoting the end index (optional)
#[macro_export]
macro_rules! convert_indices {
    ($this:ident, $start:ident, $end:ident) => {
        let len = $this.length()?;
        let $end = if $end.get_type() == "NoneType" {
            len
        } else {
            $end.to_int()?
        };
        let $start = if $start.get_type() == "NoneType" {
            0
        } else {
            $start.to_int()?
        };
        let $end = if $end < 0 { $end + len } else { $end };
        let $start = if $start < 0 { $start + len } else { $start };
        let $end = if $end < 0 {
            0
        } else {
            if $end > len {
                len as usize
            } else {
                $end as usize
            }
        };
        let $start = if $start < 0 {
            0
        } else {
            if $start > len {
                len as usize
            } else {
                $start as usize
            }
        };
    };
    ($this:ident, $start:ident) => {
        let len = $this.length()?;
        let $start = if $start.get_type() == "NoneType" {
            0
        } else {
            $start.to_int()?
        };
        let $start = if $start < 0 { $start + len } else { $start };
        let $start = if $start < 0 {
            0
        } else {
            if $start > len {
                len as usize
            } else {
                $start as usize
            }
        };
    };
}

#[cfg(test)]
mod tests {
    use crate::environment::Environment;
    use crate::values::none::NoneType;
    use crate::values::Value;

    #[test]
    fn no_arg() {
        starlark_module! { global =>
            nop() {
                Ok(Value::new(NoneType::None))
            }
        }

        let env = global(Environment::new("root"));
        env.get("nop").unwrap();
    }
}
