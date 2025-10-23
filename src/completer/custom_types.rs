use std::collections::{BTreeMap, HashMap};

/// Syntax style for method calls
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxStyle {
    /// Dot notation: self.move_to()
    Dot,
    /// Colon notation: self:move_to()
    Colon,
}

impl Default for SyntaxStyle {
    fn default() -> Self {
        Self::Dot
    }
}

/// Trait for defining custom types with their completion information
/// Implement this trait on your types to provide autocomplete support
///
/// # Example
/// ```
/// struct MyCharacter;
///
/// impl CustomType for MyCharacter {
///     fn type_name() -> &'static str {
///         "self"
///     }
///     
///     fn build_completions() -> Vec<(&'static str, &'static str, &'static str)> {
///         vec![
///             ("move_to(..)", "move_to($)", "Moves the character"),
///             ("get_health()", "get_health()", "Returns current health"),
///         ]
///     }
/// }
/// ```
pub trait CustomType {
    /// The name of the type (e.g., "self", "player", "world")
    fn type_name() -> &'static str;

    /// Build the list of completions for this type
    /// Returns a vector of (display_name, snippet, documentation) tuples
    fn build_completions() -> Vec<(&'static str, &'static str, &'static str)>;

    /// The syntax style for this type (Dot or Colon)
    /// Defaults to Dot if not overridden
    fn syntax_style() -> SyntaxStyle {
        SyntaxStyle::Dot
    }
}

/// Represents a completion item with optional snippet and documentation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompletionItem {
    /// The full text to display and match against
    pub display: String,
    /// The snippet to insert when Tab is pressed (if None, uses display)
    pub snippet: Option<String>,
    /// Documentation to show in popup (supports markdown-like formatting)
    pub documentation: Option<String>,
}

impl CompletionItem {
    pub fn new(display: impl Into<String>) -> Self {
        Self {
            display: display.into(),
            snippet: None,
            documentation: None,
        }
    }

    pub fn with_snippet(display: impl Into<String>, snippet: impl Into<String>) -> Self {
        Self {
            display: display.into(),
            snippet: Some(snippet.into()),
            documentation: None,
        }
    }

    pub fn with_snippet_and_docs(
        display: impl Into<String>,
        snippet: impl Into<String>,
        documentation: impl Into<String>,
    ) -> Self {
        Self {
            display: display.into(),
            snippet: Some(snippet.into()),
            documentation: Some(documentation.into()),
        }
    }

    pub fn with_docs(display: impl Into<String>, documentation: impl Into<String>) -> Self {
        Self {
            display: display.into(),
            snippet: None,
            documentation: Some(documentation.into()),
        }
    }

    /// Get the text to insert (snippet if available, otherwise display)
    pub fn insert_text(&self) -> &str {
        self.snippet.as_deref().unwrap_or(&self.display)
    }

    /// Check if this item has a cursor position marker ($)
    pub fn has_cursor_marker(&self) -> bool {
        self.insert_text().contains('$')
    }

    /// Get the cursor offset (position of $) and the text without $
    pub fn cursor_info(&self) -> (String, Option<usize>) {
        let text = self.insert_text();
        if let Some(pos) = text.find('$') {
            let without_marker = text.replace('$', "");
            (without_marker, Some(pos))
        } else {
            (text.to_string(), None)
        }
    }
}

/// Extension to the Completer for custom type support
#[derive(Default, Debug, Clone, PartialEq)]
pub struct CustomTypeRegistry {
    /// Maps type names (like "self") to their available methods/properties
    pub types: HashMap<String, TypeInfo>,
    /// Global completions (not tied to a type)
    pub globals: BTreeMap<String, CompletionItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeInfo {
    pub items: BTreeMap<String, CompletionItem>,
    pub syntax_style: SyntaxStyle,
}

impl CustomTypeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a type that implements the CustomType trait
    ///
    /// # Example
    /// ```
    /// registry.register_trait_type::<MyCharacter>();
    /// ```
    pub fn register_trait_type<T: CustomType>(&mut self) {
        let type_name = T::type_name();
        let completions = T::build_completions();
        let syntax_style = T::syntax_style();
        self.register_type_with_snippets_and_style(type_name, completions, syntax_style);
    }

    /// Register a type with simple method names (no snippets)
    pub fn register_type_simple(&mut self, type_name: impl Into<String>, methods: Vec<String>) {
        let type_name = type_name.into();
        let methods_map = methods
            .into_iter()
            .map(|m| (m.clone(), CompletionItem::new(m)))
            .collect();

        self.types.insert(
            type_name,
            TypeInfo {
                items: methods_map,
                syntax_style: SyntaxStyle::Dot,
            },
        );
    }

    /// Register a type with snippet and documentation support (uses Dot syntax by default)
    /// Each method is (name, snippet, docs) where snippet can include $ for cursor position
    ///
    /// Example:
    /// ```
    /// registry.register_type_with_snippets(
    ///     "self",
    ///     vec![
    ///         ("move_to", "move_to($x, y)", "Moves the character to the specified position.\n\nParameters:\n- x: X coordinate\n- y: Y coordinate"),
    ///         ("set_health", "set_health($value)", "Sets the character's health.\n\nParameters:\n- value: New health value (0-100)"),
    ///     ],
    ///     vec![],
    /// );
    /// ```
    pub fn register_type_with_snippets(
        &mut self,
        type_name: impl Into<String>,
        methods: Vec<(&str, &str, &str)>,
    ) {
        self.register_type_with_snippets_and_style(type_name, methods, SyntaxStyle::Dot);
    }

    /// Register a type with snippet and documentation support with explicit syntax style
    /// Each method is (name, snippet, docs) where snippet can include $ for cursor position
    ///
    /// Example:
    /// ```
    /// registry.register_type_with_snippets_and_style(
    ///     "self",
    ///     vec![
    ///         ("move_to", "move_to($x, y)", "Moves the character"),
    ///     ],
    ///     SyntaxStyle::Colon,
    /// );
    /// ```
    pub fn register_type_with_snippets_and_style(
        &mut self,
        type_name: impl Into<String>,
        methods: Vec<(&str, &str, &str)>,
        syntax_style: SyntaxStyle,
    ) {
        let type_name = type_name.into();
        let methods_map = methods
            .into_iter()
            .map(|(name, snippet, docs)| {
                (
                    name.to_string(),
                    CompletionItem::with_snippet_and_docs(name, snippet, docs),
                )
            })
            .collect();

        self.types.insert(
            type_name,
            TypeInfo {
                items: methods_map,
                syntax_style,
            },
        );
    }

    /// Register a type with only snippets (no docs)
    /// Each method is (name, snippet)
    ///
    /// Example:
    /// ```
    /// registry.register_type_snippets(
    ///     "self",
    ///     vec![
    ///         ("move_to", "move_to($x, y)"),
    ///         ("attack", "attack($target)"),
    ///     ],
    ///     vec![],
    /// );
    /// ```
    pub fn register_type_snippets(
        &mut self,
        type_name: impl Into<String>,
        methods: Vec<(&str, &str)>,
    ) {
        let type_name = type_name.into();
        let methods_map = methods
            .into_iter()
            .map(|(name, snippet)| {
                (
                    name.to_string(),
                    CompletionItem::with_snippet(name, snippet),
                )
            })
            .collect();

        self.types.insert(
            type_name,
            TypeInfo {
                items: methods_map,
                syntax_style: SyntaxStyle::Dot,
            },
        );
    }

    /// Register a type with only documentation (no snippets)
    /// Each method is (name, docs)
    ///
    /// Example:
    /// ```
    /// registry.register_type_docs(
    ///     "self",
    ///     vec![
    ///         ("move_to", "Moves the character"),
    ///         ("attack", "Attacks a target"),
    ///     ],
    ///     vec![],
    /// );
    /// ```
    pub fn register_type_docs(&mut self, type_name: impl Into<String>, methods: Vec<(&str, &str)>) {
        let type_name = type_name.into();
        let methods_map = methods
            .into_iter()
            .map(|(name, docs)| (name.to_string(), CompletionItem::with_docs(name, docs)))
            .collect();

        self.types.insert(
            type_name,
            TypeInfo {
                items: methods_map,
                syntax_style: SyntaxStyle::Dot,
            },
        );
    }

    /// Register global completions (like 'foreach', 'if', etc.) with full options
    ///
    /// Example:
    /// ```
    /// // With snippet and docs
    /// registry.register_global(
    ///     "foreach",
    ///     Some("for $item in items {\n    \n}"),
    ///     Some("Iterates over each item in a collection.")
    /// );
    ///
    /// // Just snippet, no docs
    /// registry.register_global("if", Some("if $condition {\n}"), None);
    ///
    /// // Just docs, no snippet
    /// registry.register_global("self", None, Some("The character instance"));
    /// ```
    pub fn register_global(
        &mut self,
        name: impl Into<String>,
        snippet: Option<impl Into<String>>,
        documentation: Option<impl Into<String>>,
    ) {
        let name_str = name.into();
        let item = match (snippet, documentation) {
            (Some(s), Some(d)) => CompletionItem::with_snippet_and_docs(&name_str, s, d),
            (Some(s), None) => CompletionItem::with_snippet(&name_str, s),
            (None, Some(d)) => CompletionItem::with_docs(&name_str, d),
            (None, None) => CompletionItem::new(&name_str),
        };
        self.globals.insert(name_str, item);
    }

    /// Register a simple global without snippet or docs
    pub fn register_global_simple(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.globals.insert(name.clone(), CompletionItem::new(name));
    }

    /// Register a global with only a snippet
    pub fn register_global_snippet(&mut self, name: impl Into<String>, snippet: impl Into<String>) {
        let name_str = name.into();
        self.globals.insert(
            name_str.clone(),
            CompletionItem::with_snippet(&name_str, snippet),
        );
    }

    /// Register a global with only documentation
    pub fn register_global_docs(
        &mut self,
        name: impl Into<String>,
        documentation: impl Into<String>,
    ) {
        let name_str = name.into();
        self.globals.insert(
            name_str.clone(),
            CompletionItem::with_docs(&name_str, documentation),
        );
    }

    /// Register a global with snippet and documentation
    pub fn register_global_snippet_docs(
        &mut self,
        name: impl Into<String>,
        snippet: impl Into<String>,
        documentation: impl Into<String>,
    ) {
        let name_str = name.into();
        self.globals.insert(
            name_str.clone(),
            CompletionItem::with_snippet_and_docs(&name_str, snippet, documentation),
        );
    }

    /// Check if any registered type uses colon syntax
    pub fn has_colon_syntax(&self) -> bool {
        self.types
            .values()
            .any(|info| info.syntax_style == SyntaxStyle::Colon)
    }

    /// Get completions for a given prefix
    /// Returns (display_text, completion_item)
    pub fn get_completions(&self, prefix: &str) -> Vec<(String, CompletionItem)> {
        let mut results = Vec::new();

        // Check if we're completing a member access (e.g., "self.move" or "self:move")
        // Try both separators
        let separator_and_type = prefix
            .rsplit_once('.')
            .map(|(t, m)| (t, m, '.'))
            .or_else(|| prefix.rsplit_once(':').map(|(t, m)| (t, m, ':')));

        if let Some((type_part, method_prefix, separator)) = separator_and_type {
            let type_name = type_part.trim();

            if let Some(type_info) = self.types.get(type_name) {
                // Determine the correct separator for this type
                let correct_separator = match type_info.syntax_style {
                    SyntaxStyle::Dot => '.',
                    SyntaxStyle::Colon => ':',
                };

                // Add methods that match the prefix
                for (method_name, item) in &type_info.items {
                    if method_prefix.is_empty() || method_name.starts_with(method_prefix) {
                        let display = format!("{}{}{}", type_name, correct_separator, method_name);
                        results.push((display, item.clone()));
                    }
                }

                return results;
            }
        }

        // Check type names (e.g., "sel" -> "self")
        for type_name in self.types.keys() {
            if type_name.starts_with(prefix) {
                results.push((type_name.clone(), CompletionItem::new(type_name)));
            }
        }

        // Check globals
        for (name, item) in &self.globals {
            if name.starts_with(prefix) {
                results.push((name.clone(), item.clone()));
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_item_cursor() {
        let item = CompletionItem::with_snippet("move_to", "move_to($x, y)");
        assert!(item.has_cursor_marker());

        let (text, pos) = item.cursor_info();
        assert_eq!(text, "move_to(x, y)");
        assert_eq!(pos, Some(8)); // Position of $ in "move_to($x, y)"
    }

    #[test]
    fn test_custom_type_completions() {
        let mut registry = CustomTypeRegistry::new();
        registry.register_type_with_snippets(
            "self",
            vec![
                ("move_to", "move_to($x, y)", "Moves character to position"),
                ("get_position", "get_position()", "Gets current position"),
            ],
        );

        // Test type name completion
        let completions = registry.get_completions("sel");
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].0, "self");

        // Test method completion
        let completions = registry.get_completions("self.");
        assert_eq!(completions.len(), 2);

        // Test partial method completion
        let completions = registry.get_completions("self.move");
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].0, "self.move_to");
        assert_eq!(completions[0].1.insert_text(), "move_to($x, y)");
    }

    #[test]
    fn test_colon_syntax() {
        let mut registry = CustomTypeRegistry::new();
        registry.register_type_with_snippets_and_style(
            "self",
            vec![
                ("move_to", "move_to($x, y)", "Moves character to position"),
                ("get_position", "get_position()", "Gets current position"),
            ],
            SyntaxStyle::Colon,
        );

        // Test type name completion
        let completions = registry.get_completions("sel");
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].0, "self");

        // Test method completion with colon
        let completions = registry.get_completions("self:");
        assert_eq!(completions.len(), 2);

        // Test partial method completion with colon
        let completions = registry.get_completions("self:move");
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].0, "self:move_to");
        assert_eq!(completions[0].1.insert_text(), "move_to($x, y)");
    }

    #[test]
    fn test_global_completions() {
        let mut registry = CustomTypeRegistry::new();
        registry.register_global_snippet("foreach", "for $item in items {\n    \n}");
        registry.register_global_snippet("if", "if $condition {\n    \n}");

        let completions = registry.get_completions("for");
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].0, "foreach");

        let completions = registry.get_completions("i");
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].0, "if");
    }

    #[test]
    fn test_global_with_options() {
        let mut registry = CustomTypeRegistry::new();

        // Test all option combinations
        registry.register_global("simple", None::<String>, None::<String>);
        registry.register_global("with_snippet", Some("snippet $here"), None::<String>);
        registry.register_global("with_docs", None::<String>, Some("Documentation"));
        registry.register_global("full", Some("snippet"), Some("docs"));

        let completions = registry.get_completions("s");
        assert!(completions.len() >= 2); // Should find "simple" and "with_snippet"
    }

    #[test]
    fn test_documentation() {
        let mut registry = CustomTypeRegistry::new();
        registry.register_type_with_snippets(
            "self",
            vec![("method", "method()", "This is documentation")],
        );

        let completions = registry.get_completions("self.");
        assert_eq!(completions.len(), 1);
        assert_eq!(
            completions[0].1.documentation,
            Some("This is documentation".to_string())
        );
    }

    #[test]
    fn test_trait_type_registration() {
        struct TestCharacter;

        impl CustomType for TestCharacter {
            fn type_name() -> &'static str {
                "player"
            }

            fn build_completions() -> Vec<(&'static str, &'static str, &'static str)> {
                vec![
                    ("jump()", "jump()", "Makes the player jump"),
                    ("attack(..)", "attack($target)", "Attacks a target"),
                ]
            }
        }

        let mut registry = CustomTypeRegistry::new();
        registry.register_trait_type::<TestCharacter>();

        // Test type name completion
        let completions = registry.get_completions("play");
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].0, "player");

        // Test method completion
        let completions = registry.get_completions("player.");
        assert_eq!(completions.len(), 2);

        // Test specific method
        let completions = registry.get_completions("player.jump");
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].0, "player.jump()");
        assert_eq!(
            completions[0].1.documentation,
            Some("Makes the player jump".to_string())
        );
    }

    #[test]
    fn test_trait_type_with_colon_syntax() {
        struct LuaCharacter;

        impl CustomType for LuaCharacter {
            fn type_name() -> &'static str {
                "self"
            }

            fn build_completions() -> Vec<(&'static str, &'static str, &'static str)> {
                vec![
                    ("move_to(..)", "move_to($x, y)", "Moves the character"),
                    ("get_health()", "get_health()", "Returns current health"),
                ]
            }

            fn syntax_style() -> SyntaxStyle {
                SyntaxStyle::Colon
            }
        }

        let mut registry = CustomTypeRegistry::new();
        registry.register_trait_type::<LuaCharacter>();

        // Test method completion with colon
        let completions = registry.get_completions("self:");
        assert_eq!(completions.len(), 2);

        // Test specific method with colon
        let completions = registry.get_completions("self:move");
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].0, "self:move_to(..)");
        assert_eq!(completions[0].1.insert_text(), "move_to($x, y)");
    }
}
