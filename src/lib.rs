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
//! use template_nest::{TemplateNest, TemplateNestOption};
//! use serde_json::json;
//!
//! let nest = TemplateNest::new(TemplateNestOption {
//!     directory: "templates".into(), ..Default::default()
//! }).unwrap();
//!
//! let simple_page = json!({
//!     "TEMPLATE": "00-simple-page",
//!     "variable": "Simple Variable",
//!     "simple_component":  {
//!         "TEMPLATE":"01-simple-component",
//!         "variable": "Simple Variable in Simple Component"
//!     }
//! });
//! println!("{}", nest.render(&simple_page).unwrap());
//! ```

use html_escape::encode_safe;
use regex::Regex;
use serde_json::Value;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fs, io,
    path::{Path, PathBuf},
    time::SystemTime,
};
use thiserror::Error;
use walkdir::WalkDir;

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

/// Options for TemplateNest.
pub struct TemplateNestOption {
    /// Delimiters used in the template. It is a tuple of two strings,
    /// representing the start and end delimiters.
    pub delimiters: (String, String),

    /// Name label used to identify the template to be used.
    pub label: String,

    /// Template extension, appended on label to identify the template.
    pub extension: String,

    /// Directory where templates are located.
    pub directory: PathBuf,

    /// Prepend & Append a string to every template which is helpful in
    /// identifying which template the output text came from.
    pub show_labels: bool,

    /// Used in conjunction with show_labels. If the template is HTML then use
    /// '<!--', '-->'.
    pub comment_delimiters: (String, String),

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
    pub token_escape_char: String,

    /// Provide a hash of default values that are substituted if template hash
    /// does not provide a value.
    pub defaults: HashMap<String, Value>,

    /// If True, then all Value::String() input is escaped. Default: True
    pub escape_html: bool,
}

/// Renders a template hash to produce an output.
pub struct TemplateNest {
    option: TemplateNestOption,

    /// Stores the indexed file in memory.
    cache: HashMap<String, TemplateFileIndex>,
}

/// Represents an indexed template file.
#[derive(Debug, Clone)]
struct TemplateFileIndex {
    /// Contents of the file.
    contents: String,

    last_modified: SystemTime,

    /// Variables in the template file.
    variables: Vec<TemplateFileVariable>,

    /// Variable names in the template file.
    variable_names: HashSet<String>,
}

/// Represents the variables in a template file.
#[derive(Debug, Clone)]
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

impl Default for TemplateNestOption {
    fn default() -> Self {
        TemplateNestOption {
            label: "TEMPLATE".to_string(),
            extension: "html".to_string(),
            show_labels: false,
            fixed_indent: false,
            die_on_bad_params: false,
            directory: "templates".into(),
            delimiters: ("<!--%".to_string(), "%-->".to_string()),
            comment_delimiters: ("<!--".to_string(), "-->".to_string()),
            token_escape_char: "".to_string(),
            defaults: HashMap::new(),
            escape_html: true,
        }
    }
}

impl TemplateNest {
    pub fn new(option: TemplateNestOption) -> Result<Self, TemplateNestError> {
        if !option.directory.is_dir() {
            return Err(TemplateNestError::TemplateDirNotFound(
                option.directory.display().to_string(),
            ));
        }

        let mut cache = HashMap::new();
        for entry in WalkDir::new(&option.directory)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| match e.metadata() {
                Ok(m) => {
                    // entry must be a file and the file name must end with option.extension
                    m.is_file() && e.file_name().to_string_lossy().ends_with(&option.extension)
                }
                Err(_) => false,
            })
        {
            let file_name = entry
                .path()
                .strip_prefix(&option.directory)
                .unwrap()
                .to_string_lossy();

            let file_name = if option.extension.is_empty() {
                &file_name
            } else {
                file_name
                    .strip_suffix(&format!(".{}", &option.extension))
                    .unwrap()
            };

            // Index the templates and store in cache.
            cache.insert(file_name.to_string(), Self::index(&option, entry.path())?);
        }

        Ok(Self { option, cache })
    }

    fn template_name_to_file(option: &TemplateNestOption, template_name: &str) -> PathBuf {
        let file_name = if option.extension.is_empty() {
            template_name.to_string()
        } else {
            format!("{}.{}", template_name, option.extension)
        };

        option.directory.join(file_name)
    }

    /// Given a template name, returns the "index" of the template file, it
    /// contains the contents of the file and all the variables that are
    /// present.
    fn index(
        option: &TemplateNestOption,
        template_file: &Path,
    ) -> Result<TemplateFileIndex, TemplateNestError> {
        if !template_file.is_file() {
            return Err(TemplateNestError::TemplateFileNotFound(
                template_file.display().to_string(),
            ));
        }

        let contents = match fs::read_to_string(&template_file) {
            Ok(file_contents) => file_contents,
            Err(err) => {
                return Err(TemplateNestError::TemplateFileReadError(err));
            }
        };

        let mut variable_names = HashSet::new();
        let mut variables = vec![];
        // Capture all the variables in the template.
        let re = Regex::new(&format!(
            "{}(.+?){}",
            option.delimiters.0, option.delimiters.1
        ))
        .unwrap();
        for cap in re.captures_iter(&contents) {
            let whole_capture = cap.get(0).unwrap();
            let start_position = whole_capture.start();

            // If token_escape_char is set then look behind for it and if we
            // find the escape char then we're only going to remove the escape
            // char and not remove this variable.
            //
            // The variable can be at the beginning of the file, that will mean
            // calculating escape_char_start results in an overflow.
            if !option.token_escape_char.is_empty()
                && start_position > option.token_escape_char.len()
            {
                let escape_char_start = start_position - option.token_escape_char.len();
                if contents[escape_char_start..start_position] == option.token_escape_char {
                    variables.push(TemplateFileVariable {
                        indent_level: 0,
                        name: "".to_string(),
                        escaped_token: true,
                        start_position: escape_char_start,
                        end_position: escape_char_start + option.token_escape_char.len(),
                    });
                    continue;
                }
            }

            // If fixed_indent is enable then record the indent level for this
            // variable. To get the indent level we look at each character in
            // reverse from the start position of the variable until we find a
            // newline character.
            let indent_level = match option.fixed_indent {
                true => {
                    // If we do not encounter a newline then that means this
                    // variable is on the first line, we take the start_position
                    // as the indent_level.
                    let newline_position = &contents[..start_position].rfind('\n');
                    match newline_position {
                        Some(position) => start_position - position - 1,
                        None => start_position,
                    }
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

        let last_modified = template_file.metadata().unwrap().modified().unwrap();
        let file_index = TemplateFileIndex {
            variable_names,
            contents,
            variables,
            last_modified,
        };
        Ok(file_index)
    }

    /// Given a TemplateHash, it parses the TemplateHash and renders a String
    /// output.
    pub fn render(&self, to_render: &Value) -> Result<String, TemplateNestError> {
        match to_render {
            Value::Null => Ok("".to_string()),
            Value::Bool(x) => Ok(x.to_string()),
            Value::String(x) => Ok(x.to_string()),
            Value::Number(x) => Ok(x.to_string()),
            Value::Array(t_array) => {
                let mut render = "".to_string();
                for t in t_array {
                    render.push_str(&self.render(t)?);
                }
                Ok(render)
            }
            Value::Object(t_hash) => {
                let t_label: &Value =
                    t_hash
                        .get(&self.option.label)
                        .ok_or(TemplateNestError::NoNameLabel(
                            self.option.label.to_string(),
                        ))?;

                // template name/path must contain a string.
                let t_path = match t_label {
                    Value::String(path) => path,
                    _ => {
                        return Err(TemplateNestError::InvalidNameLabel(
                            self.option.label.to_string(),
                        ))
                    }
                };

                let t_file = Self::template_name_to_file(&self.option, t_path);
                let t_index: Cow<TemplateFileIndex> = match self.cache.get(t_path) {
                    Some(index) => {
                        // If the file has been modified then get the latest
                        // index.
                        let last_modified = t_file.metadata().unwrap().modified().unwrap();

                        if last_modified > index.last_modified {
                            Cow::Owned(Self::index(&self.option, &t_file.as_path())?)
                        } else {
                            Cow::Borrowed(index)
                        }
                    }
                    None => Cow::Owned(Self::index(&self.option, &t_file.as_path())?),
                };

                if self.option.die_on_bad_params {
                    for var_name in t_hash.keys() {
                        // If a variable in t_hash is not present in the
                        // template file and it's not the template label then
                        // it's a bad param.
                        if !t_index.variable_names.contains(var_name)
                            && var_name != &self.option.label
                        {
                            return Err(TemplateNestError::BadParams(var_name.to_string()));
                        }
                    }
                }

                let mut rendered = String::from(&t_index.contents);

                // Iterate through all variables in reverse. We do this because
                // we don't want to mess up all the indexed positions.
                for var in t_index.variables.iter().rev() {
                    // If the variable was escaped then we just remove the
                    // token, not the variable.
                    if var.escaped_token {
                        rendered.replace_range(var.start_position..var.end_position, "");
                        continue;
                    }

                    // If the variable doesn't exist in template hash then
                    // replace it by an empty string.
                    let mut render = "".to_string();

                    // Look for the variable in t_hash, if it's not provided
                    // then we look at defaults HashMap, and then considering
                    // variable namespacing.
                    if let Some(value) = t_hash
                        .get(&var.name)
                        .or_else(|| self.option.defaults.get(&var.name))
                    {
                        let mut r: String = match value {
                            Value::String(text) => encode_safe(text).to_string(),
                            _ => self.render(value)?,
                        };

                        // If fixed_indent is set then get the indent level and
                        // replace all newlines in the rendered string.
                        if self.option.fixed_indent && var.indent_level != 0 {
                            let replacement = format!("\n{}", " ".repeat(var.indent_level));
                            r = r.replace('\n', &replacement);
                        }

                        render.push_str(&r);
                    }

                    rendered.replace_range(var.start_position..var.end_position, &render);
                }

                // Add lables to the rendered string if show_labels is true.
                if self.option.show_labels {
                    rendered.replace_range(
                        0..0,
                        &format!(
                            "{} BEGIN {} {}\n",
                            self.option.comment_delimiters.0,
                            t_path,
                            self.option.comment_delimiters.1
                        ),
                    );
                    rendered.replace_range(
                        rendered.len()..rendered.len(),
                        &format!(
                            "{} END {} {}\n",
                            self.option.comment_delimiters.0,
                            t_path,
                            self.option.comment_delimiters.1
                        ),
                    );
                }

                // Trim trailing without cloning `rendered'.
                let len_withoutcrlf = rendered.trim_end().len();
                rendered.truncate(len_withoutcrlf);

                Ok(rendered)
            }
        }
    }
}
