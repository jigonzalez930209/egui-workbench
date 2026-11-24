//! JSON-pointer patches that update a spec without rebuilding the UI.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One host patch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Patch {
    /// Set an existing value (or create a missing object key).
    Set {
        /// JSON pointer such as `/widgets/0/text`.
        path: String,
        /// New value.
        value: Value,
    },
    /// Replace an existing value.
    Replace {
        /// JSON pointer.
        path: String,
        /// New value.
        value: Value,
    },
    /// Remove an object key or array element.
    Remove {
        /// JSON pointer.
        path: String,
    },
    /// Append values to an array.
    Append {
        /// JSON pointer.
        path: String,
        /// Values to append.
        values: Vec<Value>,
    },
    /// Apply several patches in order.
    Batch {
        /// The patches.
        patches: Vec<Patch>,
    },
}

impl Patch {
    /// Apply to a JSON value.
    ///
    /// # Errors
    /// When a path is missing or the parent is not an object/array.
    pub fn apply(&self, root: &mut Value) -> Result<(), PatchError> {
        match self {
            Self::Set { path, value } | Self::Replace { path, value } => set(root, path, value.clone()),
            Self::Remove { path } => remove(root, path),
            Self::Append { path, values } => append(root, path, values.clone()),
            Self::Batch { patches } => {
                for patch in patches {
                    patch.apply(root)?;
                }
                Ok(())
            }
        }
    }
}

/// Why a patch could not be applied.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PatchError {
    /// The path does not start with `/`.
    InvalidPointer(String),
    /// The path does not exist.
    Missing(String),
    /// The target is not an array.
    NotArray(String),
    /// An array index is out of range or not numeric.
    BadIndex(String),
}

impl std::fmt::Display for PatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPointer(path) => write!(formatter, "invalid JSON pointer `{path}`"),
            Self::Missing(path) => write!(formatter, "missing path `{path}`"),
            Self::NotArray(path) => write!(formatter, "`{path}` is not an array"),
            Self::BadIndex(path) => write!(formatter, "bad array index in `{path}`"),
        }
    }
}

impl std::error::Error for PatchError {}

fn tokens(path: &str) -> Result<Vec<String>, PatchError> {
    if path.is_empty() {
        return Ok(Vec::new());
    }
    if !path.starts_with('/') {
        return Err(PatchError::InvalidPointer(path.to_owned()));
    }
    Ok(path
        .split('/')
        .skip(1)
        .map(|token| token.replace("~1", "/").replace("~0", "~"))
        .collect())
}

fn set(root: &mut Value, path: &str, value: Value) -> Result<(), PatchError> {
    let tokens = tokens(path)?;
    if tokens.is_empty() {
        *root = value;
        return Ok(());
    }
    let (last, parents) = tokens
        .split_last()
        .ok_or_else(|| PatchError::Missing(path.to_owned()))?;
    let mut current = root;
    for token in parents {
        current = child_mut(current, token, path)?;
    }
    match current {
        Value::Object(object) => {
            object.insert(last.clone(), value);
            Ok(())
        }
        Value::Array(array) => {
            let index = last
                .parse::<usize>()
                .map_err(|_| PatchError::BadIndex(path.to_owned()))?;
            let slot = array
                .get_mut(index)
                .ok_or_else(|| PatchError::Missing(path.to_owned()))?;
            *slot = value;
            Ok(())
        }
        _ => Err(PatchError::Missing(path.to_owned())),
    }
}

fn remove(root: &mut Value, path: &str) -> Result<(), PatchError> {
    let tokens = tokens(path)?;
    let (last, parents) = tokens
        .split_last()
        .ok_or_else(|| PatchError::Missing(path.to_owned()))?;
    let mut current = root;
    for token in parents {
        current = child_mut(current, token, path)?;
    }
    match current {
        Value::Object(object) => object
            .remove(last)
            .map(|_| ())
            .ok_or_else(|| PatchError::Missing(path.to_owned())),
        Value::Array(array) => {
            let index = last
                .parse::<usize>()
                .map_err(|_| PatchError::BadIndex(path.to_owned()))?;
            if index >= array.len() {
                return Err(PatchError::Missing(path.to_owned()));
            }
            array.remove(index);
            Ok(())
        }
        _ => Err(PatchError::Missing(path.to_owned())),
    }
}

fn append(root: &mut Value, path: &str, values: Vec<Value>) -> Result<(), PatchError> {
    let tokens = tokens(path)?;
    let mut current = root;
    for token in &tokens {
        current = child_mut(current, token, path)?;
    }
    let Value::Array(array) = current else {
        return Err(PatchError::NotArray(path.to_owned()));
    };
    array.extend(values);
    Ok(())
}

fn child_mut<'a>(current: &'a mut Value, token: &str, path: &str) -> Result<&'a mut Value, PatchError> {
    match current {
        Value::Object(object) => object
            .get_mut(token)
            .ok_or_else(|| PatchError::Missing(path.to_owned())),
        Value::Array(array) => {
            let index = token
                .parse::<usize>()
                .map_err(|_| PatchError::BadIndex(path.to_owned()))?;
            array.get_mut(index).ok_or_else(|| PatchError::Missing(path.to_owned()))
        }
        _ => Err(PatchError::Missing(path.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn set_replace_remove_and_append() {
        let mut value = json!({"widgets": [{"text": "a"}], "items": [1]});
        Patch::Set {
            path: "/widgets/0/text".to_owned(),
            value: json!("b"),
        }
        .apply(&mut value)
        .expect("set");
        assert_eq!(value["widgets"][0]["text"], "b");
        Patch::Append {
            path: "/items".to_owned(),
            values: vec![json!(2), json!(3)],
        }
        .apply(&mut value)
        .expect("append");
        assert_eq!(value["items"], json!([1, 2, 3]));
        Patch::Remove {
            path: "/items/0".to_owned(),
        }
        .apply(&mut value)
        .expect("remove");
        assert_eq!(value["items"], json!([2, 3]));
        Patch::Batch {
            patches: vec![
                Patch::Set {
                    path: "/new".to_owned(),
                    value: json!(true),
                },
                Patch::Replace {
                    path: "/new".to_owned(),
                    value: json!(false),
                },
            ],
        }
        .apply(&mut value)
        .expect("batch");
        assert_eq!(value["new"], false);
    }

    #[test]
    fn bad_paths_are_errors() {
        let mut value = json!({"a": 1});
        assert!(
            Patch::Set {
                path: "a".to_owned(),
                value: json!(2)
            }
            .apply(&mut value)
            .is_err()
        );
        assert!(
            Patch::Append {
                path: "/a".to_owned(),
                values: vec![json!(2)]
            }
            .apply(&mut value)
            .is_err()
        );
    }
}
