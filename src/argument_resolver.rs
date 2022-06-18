use crate::schema::{Argument, PathStr};
use dirs::home_dir;
use regex::Regex;

pub trait ArgumentResolver {
    fn resolve_argument(&self, argument: &Argument) -> String {
        match argument {
            Argument::Path { path: path_string } => self.resolve_path_argument(path_string),
            Argument::String(string) => string.clone(),
        }
    }

    fn resolve_path_argument(&self, path_string: &PathStr) -> String {
        let re = Regex::new("^~/").unwrap();
        if re.is_match(&path_string.0) {
            expand_tilda(&path_string.0)
        } else {
            path_string.0.clone()
        }
    }
}

fn expand_tilda(path_string: &str) -> String {
    let without_tilda = path_string.to_owned().split_off(2);
    let mut expanded = home_dir().unwrap();
    expanded.push(without_tilda);

    expanded.to_string_lossy().into_owned()
}
