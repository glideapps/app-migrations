/// Template for creating new migration files
pub struct Template {
    /// Template name (e.g., "bash", "ts")
    pub name: &'static str,
    /// File extension (e.g., ".sh", ".ts")
    pub extension: &'static str,
    /// Template content
    pub content: &'static str,
}

pub static TEMPLATES: &[Template] = &[
    Template {
        name: "bash",
        extension: ".sh",
        content: include_str!("../templates/bash.sh"),
    },
    Template {
        name: "ts",
        extension: ".ts",
        content: include_str!("../templates/typescript.ts"),
    },
    Template {
        name: "python",
        extension: ".py",
        content: include_str!("../templates/python.py"),
    },
    Template {
        name: "node",
        extension: ".js",
        content: include_str!("../templates/node.js"),
    },
    Template {
        name: "ruby",
        extension: ".rb",
        content: include_str!("../templates/ruby.rb"),
    },
];

/// Get a template by name
pub fn get_template(name: &str) -> Option<&'static Template> {
    TEMPLATES.iter().find(|t| t.name == name)
}

/// List all available template names
pub fn list_templates() -> impl Iterator<Item = &'static str> {
    TEMPLATES.iter().map(|t| t.name)
}
