#[cfg(feature = "to_lazy_static")]
pub mod to_lazy_static;

/// Reads and transformers the env file.
/// Note: when parsing a value, it will do in this order: Try to convert it into...
///     1. i32
///     2. f32
///     3. String (always succeeds)
/// If you want a different type to be parsed, do that in the transformer trait.
pub fn read_env<T: Transform>(env_reader: &mut EnvReader<T>) {
    let raw = String::from_utf8(env_reader.env.clone())
        .unwrap();
    let env = raw
        .split('\n')
        .collect::<Vec<_>>();

    let mut comments_above_key = vec![];

    for line in env {
        let trimmed = line.trim();

        if trimmed.is_empty() && env_reader.transformer.remove_comments_if_blank_line_occurs() {
            // Remove all comments
            comments_above_key = vec![];
        }

        // Check if the line is a comment
        if trimmed.starts_with('#') {
            comments_above_key.push(trimmed.replace('#', "//"));
            continue;
        }

        // This is the actual key, split it by checking the first '='
        // Note: when split_once is stabelized, replace it with the actual code
        //let (key, value) = trimmed.split_once('=').unwrap();

        let (key, value) = {
            let split = trimmed.find('=').unwrap();

            (&trimmed[0..split], &trimmed[split + 1..])
        };

        // Check if value can be converted to i32
        let env_type = match value.parse::<i32>() {
            Ok(o) => EnvType::I32(o),
            // Conversation failed, check if it's a f32
            Err(_) => match value.parse::<f32>() {
                Ok(o) => EnvType::F32(o),
                // Also failed, make it a string
                Err(_) => EnvType::StaticStr(value.to_string())
            }
        };

        env_reader.transformer.write(comments_above_key.clone(), key, env_type);

        // Prepare for a new loop
        comments_above_key = vec![];
    }
}

pub struct EnvReader<'a, T: Transform> {
    pub env: Vec<u8>,
    pub transformer: &'a mut T
}

impl <'a, T: Transform> EnvReader<'a, T> {
    pub fn new(env: Vec<u8>, transformer: &'a mut T) -> EnvReader<'a, T> {
        EnvReader {
            env,
            transformer
        }
    }
}

pub trait CustomMap {
    fn rust_type(&self) -> String;
    fn raw_value(&self) -> String;
    fn value(&self) -> String;
    #[cfg(feature = "to_lazy_static")]
    fn transform(&self) -> String;
}

/// The different values an env file can hold
pub enum EnvType {
    I32(i32),
    I64(i64),
    I128(i128),
    U32(u32),
    U128(u128),
    F32(f32),
    F64(f64),
    USize(usize),
    StaticStr(String),
    // Implement this type if one of the defaults is not sufficient
    Custom(Box<dyn CustomMap>),
}

impl EnvType {
    /// The Rust type
    pub fn rust_type(&self) -> String {
        match self {
            EnvType::I32(_) => "i32".to_string(),
            EnvType::I64(_) => "i64".to_string(),
            EnvType::I128(_) => "i128".to_string(),
            EnvType::U32(_) => "u32".to_string(),
            EnvType::U128(_) => "u128".to_string(),
            EnvType::F32(_) => "f32".to_string(),
            EnvType::F64(_) => "f64".to_string(),
            EnvType::USize(_) => "usize".to_string(),
            EnvType::StaticStr(_) => "&'static str".to_string(),
            EnvType::Custom(c) => c.rust_type(),
        }.replace("\"", "")
    }

    /// The actual value the env property holds
    pub fn raw_value(&self) -> String {
        match self {
            EnvType::I32(val) => val.to_string(),
            EnvType::I64(val) => val.to_string(),
            EnvType::I128(val) => val.to_string(),
            EnvType::U32(val) => val.to_string(),
            EnvType::U128(val) => val.to_string(),
            EnvType::F32(val) => val.to_string(),
            EnvType::F64(val) => val.to_string(),
            EnvType::USize(val) => val.to_string(),
            EnvType::StaticStr(val) => format!("\"{}\"", val.to_string()),
            EnvType::Custom(c) => c.raw_value(),
        }
    }

    /// Adds the type if needed behind the raw value
    /// This is needed if the user wants the value 1 to be an f32. If you only type:
    /// ```compile_fail
    /// const MY_VARIABLE: f32 = 1;
    /// ```
    /// The following compile error occurs: mismatched types [E0308] expected `f32`, found `i32`
    /// Thats why the type is needed behind the value:
    /// ```
    /// const MY_VARIABLE: f32 = 1f32;
    /// ```
    pub fn value(&self) -> String {
        let ty = self.raw_value();

        match self {
            EnvType::StaticStr(_) => ty,
            EnvType::Custom(c) => c.value(),
            _ => ty + &self.rust_type()
        }
    }
}

/// Customize transformation by implementing this trait
pub trait Transform {
    /// If two comments appear but with a blank line in between them, it may mean that the above comment
    /// should be skipped, e.g.:
    /// `
    /// # This is some comment
    /// - blank line-
    /// # This is another comment
    /// `
    /// If this method returns true, the first comment will not be included when calling env_type
    /// in the comments parameter
    fn remove_comments_if_blank_line_occurs(&self) -> bool {
        true
    }

    /// Writes the output
    fn write(&mut self, comments: Vec<String>, key: &str, inferred_type: EnvType);
}

#[cfg(test)]
mod locations {
    use std::path::PathBuf;
    use std::fs::File;
    use std::io::{Read};

    pub fn src() -> PathBuf {
        std::env::current_dir().unwrap().join("src")
    }

    pub fn env() -> Vec<u8> {
        include_bytes!("../.env").to_vec()
    }

    pub fn temp_rs() -> PathBuf {
        src().join("temp_rs.rs")
    }

    pub fn check_equals(to_check: &str) {
        // Check if the generated file is equal to what is expected
        let mut assert_test = String::new();
        File::open(src().join(to_check)).unwrap().read_to_string(&mut assert_test).unwrap();

        let mut temp_rs_string = String::new();
        File::open(temp_rs()).unwrap().read_to_string(&mut temp_rs_string).unwrap();

        // On windows, the left file somehow has \r inside the file but the right file doesn't
        assert_eq!(assert_test.replace('\r', ""), temp_rs_string);

        std::fs::remove_file(temp_rs()).unwrap();
    }
}


#[test]
fn test_write() {
    use crate::locations::{env, temp_rs, check_equals};
    use std::io::Write;
    use std::fs::File;

    // Create a transformer that writes the output to a Rust file
    struct TransformerImpl {
        file: File
    }

    impl Transform for TransformerImpl {
        fn write(&mut self, comments: Vec<String>, key: &str, inferred_type: EnvType) {
            for comment in comments {
                writeln!(&self.file, "{}", comment).unwrap();
            }

            let inferred_type = if key == "SOME_I64_VAL" {
                EnvType::I64(inferred_type.raw_value().parse().unwrap())
            } else {
                inferred_type
            };

            let declaration = format!("pub const {}: {} = {};", key, inferred_type.rust_type(), inferred_type.value());

            writeln!(&self.file, "{}", declaration).unwrap();
        }
    }

    read_env(&mut EnvReader::new(env(), &mut TransformerImpl {
        file: File::create(&temp_rs()).unwrap()
    }));

    check_equals("assert_test.rs");
}