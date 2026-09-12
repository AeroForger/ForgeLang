//! Sydrogen project configuration and source discovery.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::ast::Program;
use crate::errors::{ForgeError, ForgeResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectConfig {
    pub name: String,
    pub source_patterns: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProjectSource {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub source: String,
    pub program: Program,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub config: ProjectConfig,
    pub project_file: PathBuf,
    pub root: PathBuf,
    pub source_files: Vec<ProjectSource>,
}

impl Project {
    /// Load a project and parse every source independently, retaining its path.
    pub fn load(project_file: &Path) -> ForgeResult<Self> {
        if project_file.extension().and_then(|value| value.to_str()) != Some("blower") {
            return Err(ForgeError::parse(
                "furnace build requires a .blower project file",
            ));
        }

        let contents = fs::read_to_string(project_file).map_err(|error| {
            ForgeError::parse(format!(
                "cannot read project file '{}': {}",
                project_file.display(),
                error
            ))
        })?;
        let config = parse_project_config(&contents)?;
        let root = project_file
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let paths = discover_sources(&root, &config.source_patterns)?;
        let mut source_files = Vec::with_capacity(paths.len());

        for path in paths {
            let source = fs::read_to_string(&path).map_err(|error| {
                ForgeError::parse(format!(
                    "cannot read source '{}': {}",
                    path.display(),
                    error
                ))
            })?;
            let program = crate::parser::parse_program(&source)
                .map_err(|error| ForgeError::parse(format!("{}: {}", path.display(), error)))?;
            let relative_path = path.strip_prefix(&root).unwrap_or(&path).to_path_buf();
            source_files.push(ProjectSource {
                path,
                relative_path,
                source,
                program,
            });
        }

        Ok(Self {
            config,
            project_file: project_file.to_path_buf(),
            root,
            source_files,
        })
    }

    /// Produce the project-wide AST consumed by semantic analysis and backends.
    pub fn program(&self) -> Program {
        Program {
            statements: self
                .source_files
                .iter()
                .flat_map(|source| source.program.statements.iter().cloned())
                .collect(),
        }
    }

    /// Resolve module imports into one backend-ready project program.
    pub fn resolved_program(&self) -> ForgeResult<Program> {
        crate::imports::resolve_project(&self.source_files)
    }

    pub fn output_path(&self) -> PathBuf {
        self.root.join("build").join(&self.config.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    Ident(String),
    String(String),
    LBrace,
    RBrace,
    Equals,
    Semicolon,
    Eof,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    line: usize,
}

struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    line: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
            line: 1,
        }
    }

    fn tokenize(mut self) -> ForgeResult<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            self.skip_trivia()?;
            let line = self.line;
            let kind = match self.chars.next() {
                None => TokenKind::Eof,
                Some('{') => TokenKind::LBrace,
                Some('}') => TokenKind::RBrace,
                Some('=') => TokenKind::Equals,
                Some(';') => TokenKind::Semicolon,
                Some('"') => TokenKind::String(self.string(line)?),
                Some(character) if character.is_ascii_alphabetic() || character == '_' => {
                    let mut value = String::from(character);
                    while let Some(next) = self.chars.peek() {
                        if next.is_ascii_alphanumeric() || *next == '_' {
                            value.push(self.chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    TokenKind::Ident(value)
                }
                Some(character) => {
                    return Err(syntax_error(
                        line,
                        format!("unexpected character '{}'", character),
                    ))
                }
            };
            let eof = kind == TokenKind::Eof;
            tokens.push(Token { kind, line });
            if eof {
                return Ok(tokens);
            }
        }
    }

    fn string(&mut self, start_line: usize) -> ForgeResult<String> {
        let mut value = String::new();
        while let Some(character) = self.chars.next() {
            match character {
                '"' => return Ok(value),
                '\\' => match self.chars.next() {
                    Some('"') => value.push('"'),
                    Some('\\') => value.push('\\'),
                    Some('n') => value.push('\n'),
                    Some('t') => value.push('\t'),
                    Some(other) => {
                        return Err(syntax_error(
                            self.line,
                            format!("unsupported string escape '\\{}'", other),
                        ))
                    }
                    None => return Err(syntax_error(start_line, "unterminated string")),
                },
                '\n' => return Err(syntax_error(start_line, "unterminated string")),
                other => value.push(other),
            }
        }
        Err(syntax_error(start_line, "unterminated string"))
    }

    fn skip_trivia(&mut self) -> ForgeResult<()> {
        loop {
            while let Some(character) = self.chars.peek() {
                if character.is_whitespace() {
                    if *character == '\n' {
                        self.line += 1;
                    }
                    self.chars.next();
                } else {
                    break;
                }
            }

            let mut lookahead = self.chars.clone();
            match (lookahead.next(), lookahead.next()) {
                (Some('/'), Some('/')) => {
                    self.chars.next();
                    self.chars.next();
                    for character in self.chars.by_ref() {
                        if character == '\n' {
                            self.line += 1;
                            break;
                        }
                    }
                }
                (Some('-'), Some('[')) => {
                    self.chars.next();
                    self.chars.next();
                    let start_line = self.line;
                    let mut previous = '\0';
                    let mut closed = false;
                    for character in self.chars.by_ref() {
                        if character == '\n' {
                            self.line += 1;
                        }
                        if previous == ']' && character == '-' {
                            closed = true;
                            break;
                        }
                        previous = character;
                    }
                    if !closed {
                        return Err(syntax_error(start_line, "unterminated block comment"));
                    }
                }
                _ => return Ok(()),
            }
        }
    }
}

struct ConfigParser {
    tokens: Vec<Token>,
    position: usize,
}

impl ConfigParser {
    fn parse(mut self) -> ForgeResult<ProjectConfig> {
        let mut name: Option<String> = None;
        let mut locations: Option<Vec<String>> = None;

        while !matches!(self.peek().kind, TokenKind::Eof) {
            let section = self.ident("section name")?;
            match section.as_str() {
                "Project" => {
                    if name.is_some() {
                        return Err(self.error("duplicate Project section"));
                    }
                    name = Some(self.project_section()?);
                }
                "Files" => {
                    if locations.is_some() {
                        return Err(self.error("duplicate Files section"));
                    }
                    locations = Some(self.files_section()?);
                }
                _ => return Err(self.error(format!("unknown section '{}'", section))),
            }
        }

        let name = name.ok_or_else(|| ForgeError::parse("missing Project section"))?;
        let source_patterns =
            locations.ok_or_else(|| ForgeError::parse("missing Files section"))?;
        Ok(ProjectConfig {
            name,
            source_patterns,
        })
    }

    fn project_section(&mut self) -> ForgeResult<String> {
        self.expect(TokenKind::LBrace, "'{' after Project")?;
        let mut name = None;
        while !matches!(self.peek().kind, TokenKind::RBrace | TokenKind::Eof) {
            let field = self.ident("Project field")?;
            if field != "Name" {
                return Err(self.error(format!("unknown Project field '{}'", field)));
            }
            if name.is_some() {
                return Err(self.error("duplicate Project.Name"));
            }
            self.expect(TokenKind::Equals, "'=' after Project.Name")?;
            name = Some(self.string("Project.Name value")?);
            self.expect(TokenKind::Semicolon, "';' after Project.Name")?;
        }
        self.expect(TokenKind::RBrace, "'}' after Project section")?;
        let name = name.ok_or_else(|| ForgeError::parse("Project.Name is required"))?;
        validate_project_name(name)
    }

    fn files_section(&mut self) -> ForgeResult<Vec<String>> {
        self.expect(TokenKind::LBrace, "'{' after Files")?;
        let mut locations = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RBrace | TokenKind::Eof) {
            let field = self.ident("Files field")?;
            if field != "location" {
                return Err(self.error(format!("unknown Files field '{}'", field)));
            }
            self.expect(TokenKind::Equals, "'=' after Files.location")?;
            let value = self.string("Files.location value")?;
            if value.trim().is_empty() {
                return Err(ForgeError::parse("Files.location cannot be empty"));
            }
            locations.push(value);
            self.expect(TokenKind::Semicolon, "';' after Files.location")?;
        }
        self.expect(TokenKind::RBrace, "'}' after Files section")?;
        if locations.is_empty() {
            return Err(ForgeError::parse("Files requires at least one location"));
        }
        Ok(locations)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn ident(&mut self, expected: &str) -> ForgeResult<String> {
        let token = self.peek().clone();
        if let TokenKind::Ident(value) = token.kind {
            self.position += 1;
            Ok(value)
        } else {
            Err(syntax_error(token.line, format!("expected {}", expected)))
        }
    }

    fn string(&mut self, expected: &str) -> ForgeResult<String> {
        let token = self.peek().clone();
        if let TokenKind::String(value) = token.kind {
            self.position += 1;
            Ok(value)
        } else {
            Err(syntax_error(token.line, format!("expected {}", expected)))
        }
    }

    fn expect(&mut self, kind: TokenKind, expected: &str) -> ForgeResult<()> {
        let token = self.peek().clone();
        if token.kind == kind {
            self.position += 1;
            Ok(())
        } else {
            Err(syntax_error(token.line, format!("expected {}", expected)))
        }
    }

    fn error(&self, message: impl Into<String>) -> ForgeError {
        syntax_error(self.peek().line, message)
    }
}

fn syntax_error(line: usize, message: impl Into<String>) -> ForgeError {
    ForgeError::parse(format!(
        "invalid .blower syntax at line {}: {}",
        line,
        message.into()
    ))
}

pub fn parse_project_config(input: &str) -> ForgeResult<ProjectConfig> {
    let tokens = Lexer::new(input).tokenize()?;
    ConfigParser {
        tokens,
        position: 0,
    }
    .parse()
}

fn validate_project_name(name: String) -> ForgeResult<String> {
    if name.is_empty() {
        return Err(ForgeError::parse("Project.Name cannot be empty"));
    }
    if !name
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        return Err(ForgeError::parse(
            "Project.Name may contain only letters, numbers, '_' and '-'",
        ));
    }
    Ok(name)
}

pub fn discover_sources(root: &Path, patterns: &[String]) -> ForgeResult<Vec<PathBuf>> {
    let mut discovered = BTreeSet::new();
    for pattern in patterns {
        let normalized = pattern.replace('\\', "/");
        validate_location(&normalized)?;
        let components: Vec<&str> = normalized
            .split('/')
            .filter(|part| !part.is_empty() && *part != ".")
            .collect();
        let prefix_len = components
            .iter()
            .position(|part| part.contains('*') || part.contains('?'))
            .unwrap_or(components.len());
        let mut search_root = root.to_path_buf();
        for component in &components[..prefix_len] {
            search_root.push(component);
        }
        if prefix_len == components.len() && search_root.is_file() {
            if search_root.extension().and_then(|value| value.to_str()) == Some("anvil") {
                discovered.insert(search_root);
            }
        } else {
            let directory = if prefix_len == components.len() {
                search_root.parent().unwrap_or(root)
            } else {
                &search_root
            };
            if !directory.is_dir() {
                return Err(ForgeError::parse(format!(
                    "referenced directory does not exist for location '{}'",
                    pattern
                )));
            }
            walk_matches(root, directory, &components, &mut discovered)?;
        }

        if !discovered.iter().any(|path| {
            let relative = relative_slash_path(root, path);
            glob_matches(&components, &relative.split('/').collect::<Vec<_>>())
        }) {
            return Err(ForgeError::parse(format!(
                "no Sydrogen source files matched '{}'",
                pattern
            )));
        }
    }
    Ok(discovered.into_iter().collect())
}

fn validate_location(pattern: &str) -> ForgeResult<()> {
    let path = Path::new(pattern);
    if pattern.trim().is_empty()
        || pattern.contains('\0')
        || path.is_absolute()
        || pattern.starts_with('/')
        || pattern.split('/').any(|component| component == "..")
    {
        return Err(ForgeError::parse(format!("invalid location '{}'", pattern)));
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(ForgeError::parse(format!("invalid location '{}'", pattern)));
    }
    if pattern.contains('[')
        || pattern.contains(']')
        || pattern.contains('{')
        || pattern.contains('}')
    {
        return Err(ForgeError::parse(format!("invalid location '{}'", pattern)));
    }
    Ok(())
}

fn walk_matches(
    root: &Path,
    directory: &Path,
    pattern: &[&str],
    output: &mut BTreeSet<PathBuf>,
) -> ForgeResult<()> {
    let entries = fs::read_dir(directory).map_err(|error| {
        ForgeError::parse(format!(
            "cannot read directory '{}': {}",
            directory.display(),
            error
        ))
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| ForgeError::parse(error.to_string()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| ForgeError::parse(error.to_string()))?;
        if file_type.is_dir() {
            walk_matches(root, &path, pattern, output)?;
        } else if file_type.is_file()
            && path.extension().and_then(|value| value.to_str()) == Some("anvil")
        {
            let relative = relative_slash_path(root, &path);
            let parts: Vec<&str> = relative.split('/').collect();
            if glob_matches(pattern, &parts) {
                output.insert(path);
            }
        }
    }
    Ok(())
}

fn relative_slash_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn glob_matches(pattern: &[&str], path: &[&str]) -> bool {
    match pattern.split_first() {
        None => path.is_empty(),
        Some((&"**", remaining)) => {
            glob_matches(remaining, path) || (!path.is_empty() && glob_matches(pattern, &path[1..]))
        }
        Some((component, remaining)) => {
            !path.is_empty()
                && component_matches(component.as_bytes(), path[0].as_bytes())
                && glob_matches(remaining, &path[1..])
        }
    }
}

fn component_matches(pattern: &[u8], value: &[u8]) -> bool {
    match pattern.split_first() {
        None => value.is_empty(),
        Some((&b'*', remaining)) => {
            component_matches(remaining, value)
                || (!value.is_empty() && component_matches(pattern, &value[1..]))
        }
        Some((&b'?', remaining)) => !value.is_empty() && component_matches(remaining, &value[1..]),
        Some((&character, remaining)) => {
            value.first() == Some(&character) && component_matches(remaining, &value[1..])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn error_for(config: &str) -> String {
        parse_project_config(config).unwrap_err().to_string()
    }

    #[test]
    fn parses_valid_config_with_comments_and_multiple_locations() {
        let config = parse_project_config(
            r#"// project config
            Project { Name="Example"; }
            -[ sources ]-
            Files { location = "src/*.anvil"; location="tests/**/*.anvil"; }"#,
        )
        .unwrap();
        assert_eq!(config.name, "Example");
        assert_eq!(config.source_patterns, ["src/*.anvil", "tests/**/*.anvil"]);
    }

    #[test]
    fn validates_required_sections_and_name() {
        assert!(parse_project_config("Files { location=\"src/*.anvil\"; }")
            .unwrap_err()
            .to_string()
            .contains("missing Project"));
        assert!(parse_project_config("Project { Name=\"A\"; }")
            .unwrap_err()
            .to_string()
            .contains("missing Files"));
        assert!(
            parse_project_config("Project {} Files { location=\"src/*.anvil\"; }")
                .unwrap_err()
                .to_string()
                .contains("Name is required")
        );
        assert!(
            parse_project_config("Project { Name=\"\"; } Files { location=\"src/*.anvil\"; }")
                .unwrap_err()
                .to_string()
                .contains("cannot be empty")
        );
        assert!(parse_project_config(
            "Project { Name=\"bad/name\"; } Files { location=\"src/*.anvil\"; }"
        )
        .unwrap_err()
        .to_string()
        .contains("may contain only"));
    }

    #[test]
    fn accepts_sections_in_either_order_and_safe_project_names() {
        let config = parse_project_config(
            "Files { location=\"src/Main.anvil\"; } Project { Name=\"app_v2-64\"; }",
        )
        .unwrap();
        assert_eq!(config.name, "app_v2-64");
        assert_eq!(config.source_patterns, ["src/Main.anvil"]);
    }

    #[test]
    fn rejects_invalid_project_names() {
        for name in ["has space", "with.dot", "slash/name", "unicode-λ"] {
            let error = error_for(&format!(
                "Project {{ Name=\"{}\"; }} Files {{ location=\"src/*.anvil\"; }}",
                name
            ));
            assert!(error.contains("Project.Name may contain only"), "{error}");
        }
    }

    #[test]
    fn rejects_empty_files_section_and_location() {
        assert!(error_for("Project { Name=\"App\"; } Files {}")
            .contains("Files requires at least one location"));
        assert!(
            error_for("Project { Name=\"App\"; } Files { location=\"\"; }")
                .contains("Files.location cannot be empty")
        );
    }

    #[test]
    fn rejects_duplicates_and_malformed_syntax() {
        assert!(parse_project_config(
            "Project { Name=\"A\"; Name=\"B\"; } Files { location=\"x\"; }"
        )
        .unwrap_err()
        .to_string()
        .contains("duplicate Project.Name"));
        assert!(
            parse_project_config("Project { Name \"A\"; } Files { location=\"x\"; }")
                .unwrap_err()
                .to_string()
                .contains("invalid .blower syntax at line")
        );
    }

    #[test]
    fn rejects_duplicate_sections_and_unknown_fields() {
        let duplicate_project =
            "Project { Name=\"A\"; } Project { Name=\"B\"; } Files { location=\"x\"; }";
        assert!(error_for(duplicate_project).contains("duplicate Project section"));

        let duplicate_files =
            "Project { Name=\"A\"; } Files { location=\"x\"; } Files { location=\"y\"; }";
        assert!(error_for(duplicate_files).contains("duplicate Files section"));

        assert!(
            error_for("Project { Version=\"1\"; Name=\"A\"; } Files { location=\"x\"; }")
                .contains("unknown Project field 'Version'")
        );
        assert!(
            error_for("Project { Name=\"A\"; } Files { source=\"src/*.anvil\"; }")
                .contains("unknown Files field 'source'")
        );
        assert!(error_for(
            "Unknown { Name=\"A\"; } Project { Name=\"A\"; } Files { location=\"x\"; }"
        )
        .contains("unknown section 'Unknown'"));
    }

    #[test]
    fn malformed_input_reports_the_correct_line() {
        let missing_equals =
            "Project\n{\n    Name \"App\";\n}\nFiles { location=\"src/*.anvil\"; }";
        assert!(error_for(missing_equals).contains("line 3"));

        let unterminated_string = "Project { Name=\"App; }\nFiles { location=\"src/*.anvil\"; }";
        assert!(error_for(unterminated_string).contains("line 1: unterminated string"));

        let unterminated_comment = "-[ never closed\nProject { Name=\"App\"; }";
        assert!(error_for(unterminated_comment).contains("unterminated block comment"));
    }

    #[test]
    fn discovers_relative_globs_deduplicated_and_sorted() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("src/nested")).unwrap();
        for file in ["src/Z.anvil", "src/A.anvil", "src/nested/B.anvil"] {
            let mut output = fs::File::create(temp.path().join(file)).unwrap();
            writeln!(output, "Open Nunction Helper() {{}}").unwrap();
        }
        let paths = discover_sources(
            temp.path(),
            &["src/**/*.anvil".into(), "src/A*.anvil".into()],
        )
        .unwrap();
        let relative: Vec<_> = paths
            .iter()
            .map(|path| relative_slash_path(temp.path(), path))
            .collect();
        assert_eq!(
            relative,
            ["src/A.anvil", "src/Z.anvil", "src/nested/B.anvil"]
        );

        let dotted = discover_sources(temp.path(), &["./src/*.anvil".into()]).unwrap();
        assert_eq!(dotted.len(), 2);
    }

    #[test]
    fn exact_and_question_mark_locations_match_only_anvil_sources() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/A1.anvil"), "").unwrap();
        fs::write(temp.path().join("src/A2.anvil"), "").unwrap();
        fs::write(temp.path().join("src/A3.txt"), "").unwrap();

        let exact = discover_sources(temp.path(), &["src/A1.anvil".into()]).unwrap();
        assert_eq!(exact, [temp.path().join("src/A1.anvil")]);

        let wildcard = discover_sources(temp.path(), &["src/A?.*".into()]).unwrap();
        assert_eq!(
            wildcard,
            [
                temp.path().join("src/A1.anvil"),
                temp.path().join("src/A2.anvil")
            ]
        );
    }

    #[test]
    fn single_star_does_not_match_nested_directories() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("src/nested")).unwrap();
        fs::write(temp.path().join("src/Top.anvil"), "").unwrap();
        fs::write(temp.path().join("src/nested/Deep.anvil"), "").unwrap();

        let shallow = discover_sources(temp.path(), &["src/*.anvil".into()]).unwrap();
        assert_eq!(shallow, [temp.path().join("src/Top.anvil")]);

        let recursive = discover_sources(temp.path(), &["src/**/*.anvil".into()]).unwrap();
        assert_eq!(
            recursive,
            [
                temp.path().join("src/Top.anvil"),
                temp.path().join("src/nested/Deep.anvil")
            ]
        );
    }

    #[test]
    fn every_declared_location_must_match() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::create_dir_all(temp.path().join("tests")).unwrap();
        fs::write(temp.path().join("src/Main.anvil"), "").unwrap();
        let error = discover_sources(temp.path(), &["src/*.anvil".into(), "tests/*.anvil".into()])
            .unwrap_err()
            .to_string();
        assert!(error.contains("no Sydrogen source files matched 'tests/*.anvil'"));
    }

    #[test]
    fn project_load_retains_actual_and_relative_source_paths() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("sources")).unwrap();
        fs::write(
            temp.path().join("sources/Main.anvil"),
            "Open Nunction Main() {}",
        )
        .unwrap();
        fs::write(
            temp.path().join("app.blower"),
            "Project { Name=\"App\"; } Files { location=\"sources/*.anvil\"; }",
        )
        .unwrap();

        let project = Project::load(&temp.path().join("app.blower")).unwrap();
        assert_eq!(project.source_files.len(), 1);
        assert_eq!(
            project.source_files[0].path,
            temp.path().join("sources/Main.anvil")
        );
        assert_eq!(
            project.source_files[0].relative_path,
            PathBuf::from("sources/Main.anvil")
        );
        assert_eq!(project.output_path(), temp.path().join("build/App"));
    }

    #[test]
    fn project_program_combines_independently_parsed_source_asts() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("src")).unwrap();
        fs::write(
            temp.path().join("src/Helper.anvil"),
            "Open Int Helper() { Return 7; }",
        )
        .unwrap();
        fs::write(
            temp.path().join("src/Main.anvil"),
            "Using Helper: Helper; Open Int Main() { Return Helper(); }",
        )
        .unwrap();
        fs::write(
            temp.path().join("app.blower"),
            "Project { Name=\"App\"; } Files { location=\"src/*.anvil\"; }",
        )
        .unwrap();

        let project = Project::load(&temp.path().join("app.blower")).unwrap();
        assert_eq!(project.source_files.len(), 2);
        let program = project.resolved_program().unwrap();
        assert_eq!(program.statements.len(), 2);
        crate::semantic::analyze(&program).unwrap();
    }

    #[test]
    fn project_load_includes_source_path_in_parse_diagnostics() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/Broken.anvil"), "not Sydrogen").unwrap();
        fs::write(
            temp.path().join("app.blower"),
            "Project { Name=\"App\"; } Files { location=\"src/*.anvil\"; }",
        )
        .unwrap();
        let error = Project::load(&temp.path().join("app.blower"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("src/Broken.anvil"), "{error}");
    }

    #[test]
    fn project_load_rejects_wrong_extension_and_missing_file() {
        assert!(Project::load(Path::new("project.txt"))
            .unwrap_err()
            .to_string()
            .contains("requires a .blower project file"));
        assert!(Project::load(Path::new("definitely-missing.blower"))
            .unwrap_err()
            .to_string()
            .contains("cannot read project file"));
    }

    #[test]
    fn reports_invalid_missing_and_zero_match_locations() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("src")).unwrap();
        assert!(discover_sources(temp.path(), &["../*.anvil".into()])
            .unwrap_err()
            .to_string()
            .contains("invalid location"));
        assert!(discover_sources(temp.path(), &["..\\*.anvil".into()])
            .unwrap_err()
            .to_string()
            .contains("invalid location"));
        assert!(discover_sources(temp.path(), &["missing/*.anvil".into()])
            .unwrap_err()
            .to_string()
            .contains("directory does not exist"));
        assert!(discover_sources(temp.path(), &["src/*.anvil".into()])
            .unwrap_err()
            .to_string()
            .contains("no Sydrogen source files matched"));
    }
}
