use crate::{EnvReader, EnvType, Transform};
use std::fs::File;

use std::io::Write;

/// This will map a env file to a Rust file where all the key values in the env are inside a lazy_static block
pub fn read_env<T: LazyTransform>(env_reader: &mut EnvReader<T>) {
    env_reader.transformer.write_custom_transformers();
    // First write some utilities
    writeln!(
        env_reader.transformer.file_to_write(),
        "{}",
        r"fn string_var(v: &str) -> String {
    std::env::var(v).unwrap()
}

macro_rules! num_var {
    ($v: ident, $num: ty) => {
        #[allow(dead_code)]
        fn $v(v: &str) -> $num {
            string_var(v).parse().unwrap()
        }
    };
}

num_var!(i32_var, i32);
num_var!(i64_var, i64);
num_var!(i128_var, i128);
num_var!(u8_var, u8);
num_var!(u32_var, u32);
num_var!(u128_var, u128);
num_var!(f32_var, f32);
num_var!(f64_var, f64);
num_var!(usize_var, usize);
num_var!(bool_var, bool);

lazy_static::lazy_static! {"
    )
    .unwrap();

    // Now do the actual mapping
    crate::read_env(env_reader);

    // Close the block
    writeln!(env_reader.transformer.file_to_write(), "{}", '}').unwrap();
}

/// Note: when implementing this trait, it is ugly but the write method on Transformer needs to call this
/// lazy_static_write method.
pub trait LazyTransform: Transform {
    fn write_comments(&self) -> bool {
        true
    }
    fn write_custom_transformers(&mut self) {}
    fn file_to_write(&mut self) -> &mut File;
    fn lazy_static_write(&mut self, comments: Vec<String>, key: &str, inferred_type: EnvType) {
        let (key, ty) = self.key_value(comments.clone(), key, inferred_type);
        let file = self.file_to_write();

        for comment in &comments {
            writeln!(file, "    {}", comment).unwrap();
        }

        // lazy static does not support &'static str, only String
        let raw_type = if let EnvType::StaticStr(_) = ty {
            "String".to_string()
        } else {
            ty.rust_type()
        };

        let transform = if let EnvType::Custom(c) = &ty {
            c.transform()
        } else {
            raw_type.to_lowercase() + "_var"
        };

        writeln!(
            file,
            "    pub static ref {}: {} = {}(\"{}\");",
            key, raw_type, transform, key
        )
        .unwrap();
    }

    fn key_value(
        &mut self,
        comments: Vec<String>,
        key: &str,
        inferred_type: EnvType,
    ) -> (String, EnvType);
}

/// Struct that makes parsing easier if parsing doesn't require customization
pub struct LazyTransformDefault {
    pub file: File,
}

impl Transform for LazyTransformDefault {
    fn write(&mut self, comments: Vec<String>, key: &str, inferred_type: EnvType) {
        self.lazy_static_write(comments, key, inferred_type)
    }
}

impl LazyTransform for LazyTransformDefault {
    fn file_to_write(&mut self) -> &mut File {
        &mut self.file
    }

    fn key_value(&mut self, _comments: Vec<String>, key: &str, inferred_type: EnvType) -> (String, EnvType) {
        (key.to_string(), inferred_type)
    }
}

#[test]
fn lazy() {
    use crate::locations::env;
    use crate::locations::{check_equals, temp_rs};

    struct LazyTransformImpl {
        file: File,
    }

    impl Transform for LazyTransformImpl {
        fn write(&mut self, comments: Vec<String>, key: &str, inferred_type: EnvType) {
            self.lazy_static_write(comments, key, inferred_type)
        }
    }

    impl LazyTransform for LazyTransformImpl {
        fn file_to_write(&mut self) -> &mut File {
            &mut self.file
        }

        fn key_value(
            &mut self,
            _comments: Vec<String>,
            key: &str,
            inferred_type: EnvType,
        ) -> (String, EnvType) {
            (key.to_string(), inferred_type)
        }
    }

    read_env(&mut EnvReader::new(
        env(),
        &mut LazyTransformImpl {
            file: File::create(&temp_rs()).unwrap(),
        },
    ));

    check_equals("assert_test_lazy.rs");
}
