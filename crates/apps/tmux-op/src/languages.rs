use std::sync::LazyLock;

pub struct Language {
    pub names: Vec<&'static str>,
    pub icon: &'static str,
}

pub static LANGUAGES: LazyLock<[Language; 7]> = LazyLock::new(|| {
    [
        Language {
            names: vec!["C"],
            icon: "",
        },
        Language {
            names: vec!["C++", "CPP"],
            icon: "󰙲",
        },
        Language {
            names: vec!["C#"],
            icon: "",
        },
        Language {
            names: vec!["Typescript", "TS"],
            icon: "󰛦",
        },
        Language {
            names: vec!["Javascript", "JS"],
            icon: "",
        },
        Language {
            names: vec!["Go"],
            icon: "󰟓",
        },
        Language {
            names: vec!["Rust"],
            icon: "󱘗",
        },
    ]
});

impl Language {
    pub fn from_name(name: &str) -> Option<&'static Language> {
        LANGUAGES.iter().find(|lang| {
            let lower_names = lang
                .names
                .iter()
                .map(|name| name.to_lowercase())
                .collect::<Vec<String>>();
            lower_names.contains(&name.to_lowercase())
        })
    }
}
