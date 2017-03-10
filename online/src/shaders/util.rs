use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

macro_rules! shader {
    ($name:ident {$($part:ident: $init:ident($($arg:expr),*)$(.$call:ident($($carg:expr),*))*),*}) => {
        pub fn $name(ctx: &Facade) -> Result<::glium::program::Program, ::glium::program::ProgramCreationError> {
            use ::shaders::util::IntoSource;
            $(let $part = BuildShader::$init($($arg),*)$(.$call($($carg),*))*.build();)*
            ::glium::program::Program::new(ctx, ::glium::program::SourceCode {
                $($part: (&$part).into_source(),)*
                .. ::glium::program::SourceCode {
                    vertex_shader: "",
                    tessellation_control_shader: None,   
                    tessellation_evaluation_shader: None, 
                    geometry_shader: None,
                    fragment_shader: "",
                }
            })
        }
    };
}

pub struct BuildShader {
    prefix: String,
    source: String,
    name: Option<String>,
}

impl BuildShader {
    pub fn file(fname: &str) -> BuildShader {
        let path = Path::new(fname);
        let mut build = BuildShader {
            prefix: String::new(),
            source: String::new(),
            name: match path.file_name() {
                Some(v) => v.to_str().map(|v| v.to_owned()),
                None => None
            }
        };
        File::open(path).expect(&("Shader \"".to_owned() + fname + "\" not found")).read_to_string(&mut build.source).unwrap();
        build
    }

    pub fn define(mut self, name: &str) -> BuildShader {
    self.prefix += &format!("#define {}\n", name);
        self
    }

    pub fn define_to<S>(mut self, name: &str, val: S) -> BuildShader
    where S: ToString {
        self.prefix += &format!("#define {} {}\n", name, val.to_string());
        self
    }

    pub fn vals<'a, M>(mut self, vals: M) -> BuildShader
    where M: IntoIterator<Item = &'a (&'static str, Option<String>)> {
        for &(ref n, ref v) in vals {
            self = match *v {
                Some(ref v) => self.define_to(n, v),
                None => self.define(n),
            };
        }
        self
    }

    pub fn build(self) -> String {
        if self.source.starts_with("#version") {
            let (ver, src) = self.source.split_at(self.source.find('\n').unwrap_or(self.source.len()));
            format!("{}\n{}#line 1\n{}", ver, self.prefix, src)
        } else {
            format!("{}#line 0\n{}", self.prefix, self.source)
        }
    }
}

pub trait IntoSource<'a, S> {
    fn into_source(&'a self) -> S;
}

impl<'a> IntoSource<'a, &'a str> for String {
    fn into_source(&'a self) -> &'a str {
        self
    }
}

impl<'a> IntoSource<'a, Option<&'a str>> for String {
    fn into_source(&'a self) -> Option<&'a str> {
        Some(self)
    }
}