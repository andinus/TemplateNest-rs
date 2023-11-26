use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use regex::Regex;

pub struct TemplateNest<'a> {
    /// Delimiters used in the template. It is a tuple of two strings,
    /// representing the start and end delimiters.
    delimiters: (&'a str, &'a str),

    /// Name label used to identify the template to be used.
    label: &'a str,

    /// Template extension, appended on label to identify the template.
    extension: &'a str,

    /// Directory where templates are located.
    directory: PathBuf,
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

/// Represents a variable in a template hash, can be a string, another template
/// hash or an array of template hash.
pub enum Filling {
    Text(String),
    List(Vec<Filling>),
    Template(HashMap<String, Filling>),
}

impl TemplateNest<'_> {
    /// Creates a new instance of TemplateNest with the specified directory.
    pub fn new(directory_str: &str) -> Result<Self, String> {
        let directory = PathBuf::from(directory_str);
        if !directory.is_dir() {
            return Err(format!("Expected directory at: {}", directory_str));
        }

        let label = &"TEMPLATE";
        let extension = &"html";
        let delimiters = ("<!--%", "%-->");

        Ok(Self {
            directory,
            delimiters,
            label,
            extension,
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
                let template_label: &Filling = template_hash
                    .get(self.label)
                    .ok_or("Expected name label in template hash")?;

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

/// Helper crate for converting types into `Filling::Text`. It's used
/// internally by the `filling!` and `filling_list!` macros.

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

#[cfg(test)]
mod tests {
    use crate::TemplateNest;
    use crate::{filling, filling_list, Filling};
    use std::collections::HashMap;

    #[test]
    fn initialize() -> Result<(), String> {
        TemplateNest::new("templates")?;
        Ok(())
    }

    #[test]
    fn render_simple_page() -> Result<(), String> {
        let nest = TemplateNest::new("templates")?;
        let simple_page = filling!(
            "TEMPLATE": "00-simple-page",
            "variable": "Simple Variable",
            "simple_component":  {
                "TEMPLATE":"01-simple-component",
                "variable": "Simple Variable in Simple Component"
            }
        );
        let simple_page_output = filling!(
            "TEMPLATE": "output/01-simple-page",
        );
        assert_eq!(
            nest.render(&simple_page)?,
            nest.render(&simple_page_output)?,
        );
        Ok(())
    }

    #[test]
    fn render_incomplete_page() -> Result<(), String> {
        let nest = TemplateNest::new("templates")?;
        let incomplete_page = filling!(
            "TEMPLATE": "00-simple-page",
            "variable": "Simple Variable",
            "simple_component":  {
                "TEMPLATE":"01-simple-component",
            }
        );
        let incomplete_page_output = filling!(
            "TEMPLATE": "output/03-incomplete-page",
        );
        assert_eq!(
            nest.render(&incomplete_page)?,
            nest.render(&incomplete_page_output)?
        );
        Ok(())
    }

    #[test]
    fn render_complex_page() -> Result<(), String> {
        let nest = TemplateNest::new("templates")?;
        let complex_page = filling!(
            "TEMPLATE": "10-complex-page",
            "title": "Complex Page",
            "pre_body": {
                "TEMPLATE": "18-styles",
            },
            "navigation": {
                "TEMPLATE": "11-navigation",
                "banner": {
                    "TEMPLATE": "12-navigation-banner",
                },
                "items": [
                    { "TEMPLATE": "13-navigation-item-00-services" },
                    { "TEMPLATE": "13-navigation-item-01-resources" },
                ]
            },
            "hero_section": {
                "TEMPLATE": "14-hero-section",
            },
            "main_content": [
                { "TEMPLATE": "15-isdc-card", },
                {
                    "TEMPLATE": "16-vb-brand-cards",
                    "cards": [
                        {
                            "TEMPLATE": "17-vb-brand-card-00",
                            "parent_classes": "p-card brand-card col-4",
                        },
                        {
                            "TEMPLATE": "17-vb-brand-card-01",
                            "parent_classes": "p-card brand-card col-4",
                        },
                        {
                            "TEMPLATE": "17-vb-brand-card-02",
                            "parent_classes": "p-card brand-card col-4",
                        },
                    ]
                }
            ],
            "post_footer": {
                "TEMPLATE": "19-scripts"
            }
        );
        let complex_page_output = filling!(
            "TEMPLATE": "output/02-complex-page",
        );
        assert_eq!(
            nest.render(&complex_page)?,
            nest.render(&complex_page_output)?
        );

        Ok(())
    }

    #[test]
    fn render_array_of_template_hash() -> Result<(), String> {
        let nest = TemplateNest::new("templates")?;
        let page = filling_list!([
            {
                "TEMPLATE": "01-simple-component",
                "variable": "This is a variable",
            }, {
                "TEMPLATE": "01-simple-component",
                "variable": "This is another variable",
            }
        ]);
        let page_output = filling!(
            "TEMPLATE": "output/13-render-with-array-of-template-hash",
        );
        assert_eq!(nest.render(&page)?, nest.render(&page_output)?);

        Ok(())
    }
}
