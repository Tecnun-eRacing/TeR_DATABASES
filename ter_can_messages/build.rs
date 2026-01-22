use std::path::PathBuf;
use std::{env, fs};

use can_dbc::Dbc;

fn main() {
    build_bindings();
    build_safe_api();
}

// Robado de su tutorial
fn build_bindings() {
    let libdir_path = PathBuf::from("../")
        .canonicalize()
        .expect("cannot canonicalize path");

    let headers_path = libdir_path.join("all.h");
    let headers_path_str = headers_path.to_str().expect("Path is not a valid string");

    cc::Build::new()
        .file(libdir_path.join("all.c"))
        //.include(&libdir_path)
        .compile("all");

    let bindings = bindgen::Builder::default()
        .header(headers_path_str)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");

    println!(
        "cargo:rerun-if-changed={}",
        libdir_path.join("all.c").display()
    );
}

fn c_type_name(input: &str) -> String {
    let mut result = String::with_capacity(input.len() + 5);
    let mut chars = input.chars().peekable();
    let mut last_char: Option<char> = None;

    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            if let Some(last) = last_char {
                if last == '_' {
                } else if !last.is_uppercase() {
                    result.push('_');
                } else {
                    if let Some(next) = chars.peek() {
                        if next.is_lowercase() {
                            result.push('_');
                        }
                    }
                }
            }
            result.extend(c.to_lowercase());
        } else {
            result.push(c);
        }

        last_char = Some(c);
    }

    result
}

pub fn snake_to_pascal(s: &str) -> String {
    let mut result = String::with_capacity(s.len());

    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

fn build_safe_api() {
    let mut generated = String::new();
    let dbc = Dbc::try_from(&*fs::read_to_string("../ALL.dbc").unwrap()).unwrap();

    generated.push_str("#[derive(core::fmt::Debug)]\npub struct Error;\n\n");

    // Big enum
    {
        generated += "#[derive(core::fmt::Debug)]\npub enum CanMessage {\n";
        dbc.messages.iter().for_each(|message| {
            let safe_name = snake_to_pascal(&c_type_name(&message.name));
            generated += &format!("    {}({}),\n", &safe_name, &safe_name);
        });
        generated += "}\n\n";

        generated += "impl CanMessage {\n";
        generated += "    pub fn from_raw(id: u32, raw_msg: &[u8]) -> Result<Self, Error> {
        match id {\n";

        dbc.messages.iter().for_each(|message| {
            let id = message.id.raw();
            let safe_name = snake_to_pascal(&c_type_name(&message.name));
            generated += &format!(
                "            {} => Ok(Self::{}({}::from_raw(raw_msg)?)),\n",
                id, &safe_name, &safe_name
            );
        });

        generated += "            _ => Err(Error),
        }
    }\n\n";

        generated += "    pub fn serialize(&self, dst: &mut [u8]) -> Result<(u32, usize), Error> {
        match self {\n";
        dbc.messages.iter().for_each(|message| {
            let id = message.id.raw();
            let safe_name = snake_to_pascal(&c_type_name(&message.name));
            generated += &format!(
                "            Self::{}(inner) => {{ let len = inner.to_raw(dst)?; Ok(({}, len)) }},\n",
                safe_name, id
            );
        });
        generated += "        }
    }\n";
        generated += "}\n\n"
    }

    // Message structs
    {
        dbc.messages.iter().for_each(|message| {
        let msg_type_name = c_type_name(&message.name);
        let safe_type_name = snake_to_pascal(&msg_type_name);
        generated += &format!(
            "#[derive(core::fmt::Debug)]\npub struct {}(crate::raw_bindings::all_{}_t);\n\n",
            &safe_type_name, &msg_type_name
        );

        generated += &format!(
            r#"impl {} {{
    pub fn new(from: crate::raw_bindings::all_{}_t) -> Self {{
        Self(from)
    }}

    pub fn from_raw(src: &[u8]) -> Result<Self, Error> {{
        let mut dst = core::mem::MaybeUninit::uninit();
        let res = unsafe {{ crate::raw_bindings::all_{}_unpack(dst.as_mut_ptr(), src.as_ptr(), src.len()) }};

        if res != 0 {{
            return Err(Error);
        }}

        Ok(Self(unsafe {{ dst.assume_init() }}))
    }}

    pub fn to_raw(&self, dst: &mut [u8]) -> Result<usize, Error> {{
        let res = unsafe {{ crate::raw_bindings::all_{}_pack(dst.as_mut_ptr(), &self.0, dst.len()) }};

        if res < 0 {{
            return Err(Error);
        }}

        Ok(res as usize)
    }}


}}

"#,
            &safe_type_name, &msg_type_name, &msg_type_name, &msg_type_name
        );

        message.signals.iter().for_each(|signal| {
            let signal_type_name = c_type_name(&signal.name);
            generated += &format!(
                r#"impl {} {{
    // Be careful as it does not check if its multiplexed for its multiplexed state
    pub fn get_{}(&self) -> f32 {{
        unsafe {{ crate::raw_bindings::all_{}_{}_decode(self.0.{}) }}
    }}

    // Be careful as it does not check if its multiplexed for its multiplexed state
    pub fn set_{}(&mut self, val: f32) {{
        self.0.{} = unsafe {{ crate::raw_bindings::all_{}_{}_encode(val) }};
    }}
}}
"#,
                &safe_type_name, &signal_type_name, &msg_type_name, &signal_type_name, &signal_type_name, &signal_type_name, &signal_type_name, &msg_type_name, &signal_type_name
                );
            });
            generated.push('\n');
        });
    }

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("safe.rs");
    fs::write(out_path, generated).unwrap();
}
