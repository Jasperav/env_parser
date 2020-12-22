fn string_var(v: &str) -> String {
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
num_var!(u32_var, u32);
num_var!(u128_var, u128);
num_var!(f32_var, f32);
num_var!(f64_var, f64);
num_var!(usize_var, usize);
num_var!(bool_var, bool);

lazy_static::lazy_static! {
    pub static ref SOME_KEY: String = string_var("SOME_KEY");
    // this is a comment
    pub static ref SOME_KEY_WITH_COMMENT: i32 = i32_var("SOME_KEY_WITH_COMMENT");
    pub static ref SOME_I64_VAL: i32 = i32_var("SOME_I64_VAL");
    pub static ref SOME_F32: f32 = f32_var("SOME_F32");
    pub static ref ANOTHER_F32: f32 = f32_var("ANOTHER_F32");
    pub static ref BOOL_TRUE: bool = bool_var("BOOL_TRUE");
}
