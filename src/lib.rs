//! Template engine for Rust.
//!
//! For more details on the idea behind `Template::Nest` read:
//! - <https://metacpan.org/pod/Template::Nest#DESCRIPTION>
//! - <https://pypi.org/project/template-nest/>

//! # Examples
//!
//! More examples in `examples/` directory.
//!
//! Place these files in `templates` directory:
//! `templates/00-simple-page.html`:
//! ```html
//! <!DOCTYPE html>
//! <html lang="en">
//!   <head>
//!     <meta charset="utf-8">
//!     <meta name="viewport" content="width=device-width, initial-scale=1">
//!     <title>Simple Page</title>
//!   </head>
//!   <body>
//!     <p>A fairly simple page to test the performance of Template::Nest.</p>
//!     <p><!--% variable %--></p>
//!     <!--% simple_component %-->
//!   </body>
//! </html>
//! ```
//!
//! `templates/00-simple-page.html`:
//! ```html
//! <p><!--% variable %--></p>
//! ```
//!
//! Those templates can be used in a template hash which is passed to
//! TemplateNest::render to render a page:
//! ```rust
//! use template_nest::TemplateNest;
//! use template_nest::{filling, Filling};
//! use std::collections::HashMap;
//!
//! let nest = TemplateNest::new("templates").unwrap();
//! let simple_page = filling!(
//!     "TEMPLATE": "00-simple-page",
//!     "variable": "Simple Variable",
//!     "simple_component":  {
//!         "TEMPLATE":"01-simple-component",
//!         "variable": "Simple Variable in Simple Component"
//!     }
//! );
//! println!("{}", nest.render(&simple_page).unwrap());
//! ```

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::{fs, io};

use regex::Regex;
use thiserror::Error;

/// Represents a variable in a template hash, can be a string, another template
/// hash or an array of template hash.
pub enum Filling {
    Text(String),
    List(Vec<Filling>),
    Template(HashMap<String, Filling>),
}

impl Filling {
    /// inserts into a Template, returns an Err if the enum if not of Template
    /// variant.
    pub fn insert(&mut self, variable: String, to_insert: Filling) -> Result<(), &'static str> {
        match self {
            Filling::Template(ref mut map) => {
                map.insert(variable, to_insert);
                Ok(())
            }
            _ => Err("Cannot insert into non-Template variant"),
        }
    }

    /// push into a List, returns an Err if the enum if not of List variant.
    pub fn push(&mut self, to_push: Filling) -> Result<(), &'static str> {
        match self {
            Filling::List(ref mut list) => {
                list.push(to_push);
                Ok(())
            }
            _ => Err("Cannot push into non-List variant"),
        }
    }
}

#[derive(Error, Debug)]
pub enum TemplateNestError {
    #[error("expected template directory at `{0}`")]
    TemplateDirNotFound(String),

    #[error("expected template file at `{0}`")]
    TemplateFileNotFound(String),

    #[error("error reading: `{0}`")]
    TemplateFileReadError(#[from] io::Error),

    #[error("encountered hash with no name label (name label: `{0}`)")]
    NoNameLabel(String),

    #[error("encountered hash with invalid name label type (name label: `{0}`)")]
    InvalidNameLabel(String),

    #[error("bad params in template hash, variable not present in template file: `{0}`")]
    BadParams(String),
}

/// Renders a template hash to produce an output.
pub struct TemplateNest<'a> {
    /// Delimiters used in the template. It is a tuple of two strings,
    /// representing the start and end delimiters.
    pub delimiters: (&'a str, &'a str),

    /// Name label used to identify the template to be used.
    pub label: &'a str,

    /// Template extension, appended on label to identify the template.
    pub extension: &'a str,

    /// Directory where templates are located.
    pub directory: PathBuf,

    /// Prepend & Append a string to every template which is helpful in
    /// identifying which template the output text came from.
    pub show_labels: bool,

    /// Used in conjunction with show_labels. If the template is HTML then use
    /// '<!--', '-->'.
    pub comment_delimiters: (&'a str, &'a str),

    /// Intended to improve readability when inspecting nested templates.
    pub fixed_indent: bool,

    /// If True, then an attempt to populate a template with a variable that
    /// doesn't exist (i.e. name not found in template file) results in an
    /// error.
    pub die_on_bad_params: bool,

    /// Escapes a token delimiter, i.e. if set to '\' then prefixing the token
    /// delimiters with '\' means it won't be considered a variable.
    ///
    /// <!--% token %-->  => is a variable
    /// \<!--% token %--> => is not a variable. ('\' is removed from output)
    pub token_escape_char: &'a str,
}

/// Represents an indexed template file.
struct TemplateFileIndex {
    /// Contents of the file.
    contents: String,

    /// Variables in the template file.
    variables: Vec<TemplateFileVariable>,

    /// Variable names in the template file.
    variable_names: HashSet<String>,
}

/// Represents the variables in a template file.
struct TemplateFileVariable {
    name: String,

    /// Start & End positions of the complete variable string. i.e. including
    /// the delimeters.
    start_position: usize,
    end_position: usize,

    /// Indent level of the variable.
    indent_level: usize,

    /// If true then this variable was escaped with token_escape_char, we just
    /// need to remove the escape character.
    escaped_token: bool,
}

impl Default for TemplateNest<'_> {
    fn default() -> Self {
        TemplateNest {
            label: "TEMPLATE",
            extension: "html",
            show_labels: false,
            fixed_indent: false,
            die_on_bad_params: false,
            directory: "templates".into(),
            delimiters: ("<!--%", "%-->"),
            comment_delimiters: ("<!--", "-->"),
            token_escape_char: "",
        }
    }
}

impl TemplateNest<'_> {
    /// Creates a new instance of TemplateNest with the specified directory.
    pub fn new(directory_str: &str) -> Result<Self, TemplateNestError> {
        let directory = PathBuf::from(directory_str);
        if !directory.is_dir() {
            return Err(TemplateNestError::TemplateDirNotFound(
                directory_str.to_string(),
            ));
        }

        Ok(Self {
            directory,
            ..Default::default()
        })
    }

    /// Given a template name, returns the "index" of the template file, it
    /// contains the contents of the file and all the variables that are
    /// present.
    fn index(&self, template_name: &str) -> Result<TemplateFileIndex, TemplateNestError> {
        let file = self
            .directory
            .join(format!("{}.{}", template_name, self.extension));
        if !file.is_file() {
            return Err(TemplateNestError::TemplateFileNotFound(
                file.display().to_string(),
            ));
        }

        let contents = match fs::read_to_string(&file) {
            Ok(file_contents) => file_contents,
            Err(err) => {
                return Err(TemplateNestError::TemplateFileReadError(err));
            }
        };

        let mut variable_names = HashSet::new();
        let mut variables = vec![];
        // Capture all the variables in the template.
        let re = Regex::new(&format!("{}(.+?){}", self.delimiters.0, self.delimiters.1)).unwrap();
        for cap in re.captures_iter(&contents) {
            let whole_capture = cap.get(0).unwrap();
            let start_position = whole_capture.start();

            // If token_escape_char is set then look behind for it and if we
            // find the escape char then we're only going to remove the escape
            // char and not remove this variable.
            //
            // The variable can be at the beginning of the file, that will mean
            // calculating escape_char_start results in an overflow.
            if !self.token_escape_char.is_empty() && start_position > self.token_escape_char.len() {
                let escape_char_start = start_position - self.token_escape_char.len();
                if &contents[escape_char_start..start_position] == self.token_escape_char {
                    variables.push(TemplateFileVariable {
                        indent_level: 0,
                        name: "".to_string(),
                        escaped_token: true,
                        start_position: escape_char_start,
                        end_position: escape_char_start + self.token_escape_char.len(),
                    });
                    continue;
                }
            }

            // If fixed_indent is enable then record the indent level for this
            // variable. To get the indent level we look at each character in
            // reverse from the start position of the variable until we find a
            // newline character.
            let indent_level = match self.fixed_indent {
                true => {
                    let newline_position = &contents[..start_position].rfind('\n').unwrap_or(0);
                    start_position - newline_position - 1
                }
                false => 0,
            };

            let variable_name = cap[1].trim();
            variable_names.insert(variable_name.to_string());
            variables.push(TemplateFileVariable {
                indent_level,
                start_position,
                end_position: whole_capture.end(),
                name: variable_name.to_string(),
                escaped_token: false,
            });
        }

        let file_index = TemplateFileIndex {
            variable_names,
            contents,
            variables,
        };
        Ok(file_index)
    }

    /// Given a TemplateHash, it parses the TemplateHash and renders a String
    /// output.
    pub fn render(&self, filling: &Filling) -> Result<String, TemplateNestError> {
        match filling {
            Filling::Text(text) => Ok(text.to_string()),
            Filling::List(list) => {
                let mut render = "".to_string();
                for f in list {
                    render.push_str(&self.render(f)?);
                }
                Ok(render)
            }
            Filling::Template(template_hash) => {
                let template_label: &Filling = template_hash
                    .get(self.label)
                    .ok_or(TemplateNestError::NoNameLabel(self.label.to_string()))?;

                // template_name must contain a string, it cannot be a template hash or
                // a vec of template hash.
                if let Filling::Text(name) = template_label {
                    let template_index = self.index(name)?;

                    // Check for bad params.
                    if self.die_on_bad_params {
                        for name in template_hash.keys() {
                            // If a variable in template_hash is not present in
                            // the template file and it's not the template label
                            // then it's a bad param.
                            if !template_index.variable_names.contains(name) && name != self.label {
                                return Err(TemplateNestError::BadParams(name.to_string()));
                            }
                        }
                    }

                    let mut rendered = String::from(&template_index.contents);

                    // Iterate through all variables in reverse. We do this because we
                    // don't want to mess up all the indexed positions.
                    for var in template_index.variables.iter().rev() {
                        // If the variable was escaped then we just remove the token, not the variable.
                        if var.escaped_token {
                            rendered.replace_range(var.start_position..var.end_position, "");
                            continue;
                        }

                        let mut render = "".to_string();

                        // If the variable doesn't exist in template hash then
                        // replace it by an empty string.
                        if let Some(value) = template_hash.get(&var.name) {
                            let mut r: String = self.render(value)?;

                            // If fixed_indent is set then get the indent level
                            // and replace all newlines in the rendered string.
                            if self.fixed_indent && var.indent_level != 0 {
                                let replacement = format!("\n{}", " ".repeat(var.indent_level));
                                r = r.replace('\n', &replacement);
                            }

                            render.push_str(&r);
                        }

                        rendered.replace_range(var.start_position..var.end_position, &render);
                    }

                    // Add lables to the rendered string if show_labels is true.
                    if self.show_labels {
                        rendered.replace_range(
                            0..0,
                            &format!(
                                "{} BEGIN {} {}\n",
                                self.comment_delimiters.0, name, self.comment_delimiters.1
                            ),
                        );
                        rendered.replace_range(
                            rendered.len()..rendered.len(),
                            &format!(
                                "{} END {} {}\n",
                                self.comment_delimiters.0, name, self.comment_delimiters.1
                            ),
                        );
                    }

                    // Trim trailing without cloning `rendered'.
                    let len_withoutcrlf = rendered.trim_end().len();
                    rendered.truncate(len_withoutcrlf);

                    Ok(rendered)
                } else {
                    Err(TemplateNestError::InvalidNameLabel(self.label.to_string()))
                }
            }
        }
    }
}

// The below macros are adapted from the json-rust macros (https://docs.rs/json/latest/json/#macros)
#[macro_export]
macro_rules! filling_list {
    //[] => ($crate::JsonValue::new_array());
    [] => { "".to_string() };

    // Handles for token tree items
    [@ITEM($( $i:expr, )*) $item:tt, $( $cont:tt )+] => {
        $crate::filling_list!(
            @ITEM($( $i, )* $crate::filling_text!($item), )
                $( $cont )*
        )
    };
    (@ITEM($( $i:expr, )*) $item:tt,) => ({
        $crate::filling_list!(@END $( $i, )* $crate::filling_text!($item), )
    });
    (@ITEM($( $i:expr, )*) $item:tt) => ({
        $crate::filling_list!(@END $( $i, )* $crate::filling_text!($item), )
    });

    // Handles for expression items
    [@ITEM($( $i:expr, )*) $item:expr, $( $cont:tt )+] => {
        $crate::filling_list!(
            @ITEM($( $i, )* $crate::filling_text!($item), )
                $( $cont )*
        )
    };
    (@ITEM($( $i:expr, )*) $item:expr,) => ({
        $crate::filling_list!(@END $( $i, )* $crate::filling_text!($item), )
    });
    (@ITEM($( $i:expr, )*) $item:expr) => ({
        $crate::filling_list!(@END $( $i, )* $crate::filling_text!($item), )
    });

    // Construct the actual array
    (@END $( $i:expr, )*) => ({
        let size = 0 $( + {let _ = &$i; 1} )*;
        let mut vec: Vec<Filling> = Vec::with_capacity(size);

        $(
            vec.push($i);
        )*

            $crate::Filling::List( vec )
    });

    // Entry point to the macro
    ($( $cont:tt )+) => {
        $crate::filling_list!(@ITEM() $($cont)*)
    };
}

/// Helper macro for converting types into `Filling::Text`. It's used internally
/// by the `filling!` and `filling_list!` macros.
#[macro_export]
macro_rules! filling_text {
    //( null ) => { $crate::Null };
    ( null ) => { "".to_string() };
    ( [$( $token:tt )*] ) => {
        // 10
        $crate::filling_list![ $( $token )* ]
    };
    ( {$( $token:tt )*} ) => {
        $crate::filling!{ $( $token )* }
    };
    { $value:expr } => { $crate::Filling::Text($value.to_string()) };
}

/// Helper macro for creating instances of `Filling`.
#[macro_export]
macro_rules! filling {
    {} => { "".to_string() };

    // Handles for different types of keys
    (@ENTRY($( $k:expr => $v:expr, )*) $key:ident: $( $cont:tt )*) => {
        $crate::filling!(@ENTRY($( $k => $v, )*) stringify!($key) => $($cont)*)
    };
    (@ENTRY($( $k:expr => $v:expr, )*) $key:literal: $( $cont:tt )*) => {
        $crate::filling!(@ENTRY($( $k => $v, )*) $key => $($cont)*)
    };
    (@ENTRY($( $k:expr => $v:expr, )*) [$key:expr]: $( $cont:tt )*) => {
        $crate::filling!(@ENTRY($( $k => $v, )*) $key => $($cont)*)
    };

    // Handles for token tree values
    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:tt, $( $cont:tt )+) => {
        $crate::filling!(
            @ENTRY($( $k => $v, )* $key => $crate::filling_text!($value), )
                $( $cont )*
        )
    };
    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:tt,) => ({
        $crate::filling!(@END $( $k => $v, )* $key => $crate::filling_text!($value), )
    });
    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:tt) => ({
        $crate::filling!(@END $( $k => $v, )* $key => $crate::filling_text!($value), )
    });

    // Handles for expression values
    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:expr, $( $cont:tt )+) => {
        $crate::filling!(
            @ENTRY($( $k => $v, )* $key => $crate::filling_text!($value), )
                $( $cont )*
        )
    };
    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:expr,) => ({
        $crate::filling!(@END $( $k => $v, )* $key => $crate::filling_text!($value), )
    });

    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:expr) => ({
        $crate::filling!(@END $( $k => $v, )* $key => $crate::filling_text!($value), )
    });

    // Construct the actual object
    (@END $( $k:expr => $v:expr, )*) => ({
        let mut params : HashMap<String, Filling> = Default::default();
        $(
            params.insert($k.to_string(), $v);
        )*
            let template = $crate::Filling::Template( params );
        template
    });

    // Entry point to the macro
    ($key:tt: $( $cont:tt )+) => {
        $crate::filling!(@ENTRY() $key: $($cont)*)
    };

    // Legacy macro
    ($( $k:expr => $v:expr, )*) => {
        $crate::filling!(@END $( $k => $crate::filling_text!($v), )*)
    };
    ($( $k:expr => $v:expr ),*) => {
        $crate::filling!(@END $( $k => $crate::filling_text!($v), )*)
    };
}
