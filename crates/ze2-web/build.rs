mod helpers {
    use std::env::VarError;

    pub fn env_opt(name: &str) -> String {
        match std::env::var(name) {
            Ok(value) => value,
            Err(VarError::NotPresent) => String::new(),
            Err(VarError::NotUnicode(_)) => {
                panic!("Environment variable `{name}` is not valid Unicode")
            }
        }
    }
}

#[path = "../ze2/build/i18n.rs"]
mod i18n;

fn main() {
    let i18n_path = "../../i18n/ze2.toml";
    let i18n = std::fs::read_to_string(i18n_path).unwrap();
    let contents = i18n::generate(&i18n);
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let path = format!("{out_dir}/i18n_ze2.rs");
    std::fs::write(&path, contents).unwrap();

    println!("cargo::rerun-if-env-changed=EDIT_CFG_LANGUAGES");
    println!("cargo::rerun-if-changed={i18n_path}");
}
