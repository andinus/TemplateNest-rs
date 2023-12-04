//! Template engine for Rust.
//!
//! For more details on the idea behind `Template::Nest` read:
//! - <https://metacpan.org/pod/Template::Nest#DESCRIPTION>
//! - <https://pypi.org/project/template-nest/>

//! # Examples
//!
//! Templates placed in `templates` directory:
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

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use regex::Regex;

/// Represents a variable in a template hash, can be a string, another template
/// hash or an array of template hash.
pub enum Filling {
    Text(String),
    List(Vec<Filling>),
    Template(HashMap<String, Filling>),
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

    /// Prepends & Appends a string to every template which is helpful in
    /// identifying which template the output text came from.
    pub show_labels: bool,

    /// Used in conjuction with show_labels. If the template is HTML then use
    /// '<!--', '-->'.
    pub comment_delimiters: (&'a str, &'a str),
}

/// Represents an indexed template file.
struct TemplateFileIndex {
    /// Contents of the file.
    contents: String,

    /// Variables in the template file.
    variables: Vec<TemplateFileVariable>,
}

/// Represents the variables in a template file.
struct TemplateFileVariable {
    name: String,

    /// Start & End positions of the complete variable string. i.e. including
    /// the delimeters.
    start_position: usize,
    end_position: usize,
}

impl Default for TemplateNest<'_> {
    fn default() -> Self {
        TemplateNest {
            label: "TEMPLATE",
            extension: "html",
            show_labels: false,
            directory: "templates".into(),
            delimiters: ("<!--%", "%-->"),
            comment_delimiters: ("<!--", "-->"),
        }
    }
}

impl TemplateNest<'_> {
    /// Creates a new instance of TemplateNest with the specified directory.
    pub fn new(directory_str: &str) -> Result<Self, String> {
        let directory = PathBuf::from(directory_str);
        if !directory.is_dir() {
            return Err(format!("Expected directory at: {}", directory_str));
        }

        Ok(Self {
            directory,
            ..Default::default()
        })
    }

    /// Given a template name, returns the "index" of the template file, it
    /// contains the contents of the file and all the variables that are
    /// present.
    fn index(&self, template_name: &str) -> Result<TemplateFileIndex, String> {
        let file = self
            .directory
            .join(format!("{}.{}", template_name, self.extension));
        if !file.is_file() {
            return Err(format!("Expected file at: {}", file.display()));
        }

        let contents = match fs::read_to_string(&file) {
            Ok(file_contents) => file_contents,
            Err(err) => {
                return Err(format!("Error reading file: {}", err));
            }
        };

        let mut variables = vec![];
        // Capture all the variables in the template.
        let re = Regex::new(&format!("{}(.+?){}", self.delimiters.0, self.delimiters.1)).unwrap();
        for cap in re.captures_iter(&contents) {
            let whole_capture = cap.get(0).unwrap();

            variables.push(TemplateFileVariable {
                name: cap[1].trim().to_string(),
                start_position: whole_capture.start(),
                end_position: whole_capture.end(),
            });
        }

        let file_index = TemplateFileIndex {
            contents,
            variables,
        };
        Ok(file_index)
    }

    /// Given a TemplateHash, it parses the TemplateHash and renders a String
    /// output.
    pub fn render(&self, filling: &Filling) -> Result<String, String> {
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
                let template_label: &Filling = template_hash.get(self.label).ok_or(format!(
                    "Expected name label in template hash: `{}`",
                    &self.label
                ))?;

                // template_name must contain a string, it cannot be a template hash or
                // a vec of template hash.
                if let Filling::Text(name) = template_label {
                    let template_index = self.index(name)?;
                    let mut rendered = String::from(&template_index.contents);

                    // Iterate through all variables in reverse. We do this because we
                    // don't want to mess up all the indexed positions.
                    for var in template_index.variables.iter().rev() {
                        let mut render = "".to_string();
                        if let Some(value) = template_hash.get(&var.name) {
                            render.push_str(&self.render(value)?);
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
                    Err("Name label should be of type Filling::Text(_)".to_string())
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
