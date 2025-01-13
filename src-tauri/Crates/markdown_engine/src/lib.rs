use html5ever::driver::ParseOpts;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{parse_document, serialize};
// use html5ever::tree_builder::TreeSink;
use markup5ever_rcdom::{RcDom, NodeData, Handle};
// use std::borrow::Cow;
use std::collections::HashMap;

// Public interface
pub fn convert_to_markdown(html: &str) -> String {
    MarkdownConverter::new().convert_to_markdown(html)
}

// Define element handlers as trait for better extensibility
pub trait ElementHandler {
    fn handle(&self, converter: &MarkdownConverter, node: &Handle, attrs: &[html5ever::Attribute], output: &mut String, depth: usize);
}

// Struct to hold element handlers
pub struct MarkdownConverter {
    handlers: HashMap<&'static str, Box<dyn ElementHandler>>,
}

impl MarkdownConverter {
    pub fn new() -> Self {
        let mut converter = Self { 
            handlers: HashMap::new() 
        };
        
        // Register handlers for different elements
        converter.handlers.insert("p", Box::new(ParagraphHandler));
        converter.handlers.insert("mark", Box::new(MarkHandler));
        converter.handlers.insert("s", Box::new(StrikeThroughHandler));
        converter.handlers.insert("em", Box::new(EmphasisHandler));
        converter.handlers.insert("b", Box::new(BoldHandler));
        converter.handlers.insert("u", Box::new(UnderlineHandler));
        converter.handlers.insert("a", Box::new(LinkHandler));
        converter.handlers.insert("span", Box::new(SpanHandler));
        converter.handlers.insert("blockquote", Box::new(BlockquoteHandler));
        converter.handlers.insert("code", Box::new(InlineCodeHandler));
        converter.handlers.insert("pre", Box::new(CodeBlockHandler));

        // Register handlers for headers
        converter.handlers.insert("h1", Box::new(HeaderHandler::level(1)));
        converter.handlers.insert("h2", Box::new(HeaderHandler::level(2)));
        converter.handlers.insert("h3", Box::new(HeaderHandler::level(3)));
        converter.handlers.insert("h4", Box::new(HeaderHandler::level(4)));
        converter.handlers.insert("h5", Box::new(HeaderHandler::level(5)));
        converter.handlers.insert("h6", Box::new(HeaderHandler::level(6)));
        
        converter
    }

    pub fn convert_to_markdown(&self, html: &str) -> String {
        println!("{}", html);
        let dom = self.parse_to_dom(html);
        let mut markdown = String::with_capacity(html.len());
        self.traverse_dom(&dom.document, &mut markdown, 0);
        markdown.trim().to_string()
    }

    fn parse_to_dom(&self, html: &str) -> RcDom {
        let opts = ParseOpts {
            tree_builder: TreeBuilderOpts {
                drop_doctype: true,
                ..Default::default()
            },
            ..Default::default()
        };

        parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .unwrap()
    }

    fn traverse_dom(&self, node: &Handle, output: &mut String, depth: usize) {
        match &node.data {
            NodeData::Text { contents } => {
                let text = contents.borrow();
                if !text.trim().is_empty() {
                    output.push_str(&text);
                }
            },
            NodeData::Element { name, attrs, .. } => {
                let tag_name = name.local.as_ref();

                // Use registered handler if available
                if let Some(handler) = self.handlers.get(tag_name) {
                    handler.handle(self, node, &attrs.borrow(), output, depth);
                } else {
                    // Default behavior for unknown elements
                    self.walk_children(node, output, depth);
                }
            },
            _ => self.walk_children(node, output, depth),
        }
    }

    pub fn walk_children(&self, node: &Handle, output: &mut String, depth: usize) {
        for child in node.children.borrow().iter() {
            self.traverse_dom(child, output, depth + 1);
        }
    }
}

struct StyleParser;

impl StyleParser {
    fn parse_styles(style_str: &str) -> HashMap<String, String> {
        let mut styles = HashMap::new();
        for style in style_str.split(';') {
            if let Some((key, value)) = style.split_once(':') {
                styles.insert(
                    key.trim().to_string(),
                    value.trim().to_string()
                );
            }
        }
        styles
    }

    fn get_markdown_style(styles: &HashMap<String, String>) -> Option<(String, String)> {
        if let Some(weight) = styles.get("font-weight") {
            if weight == "bold" || weight == "700" {
                return Some(("**".to_string(), "**".to_string()));
            }
        }
        if let Some(style) = styles.get("font-style") {
            if style == "italic" {
                return Some(("*".to_string(), "*".to_string()));
            }
        }
        if let Some(decoration) = styles.get("text-decoration") {
            if decoration == "underline" {
                return Some(("__".to_string(), "__".to_string()));
            }
        }
        None
    }
}

// Implementation of handlers
// Header handler implementation
struct HeaderHandler {
    level: usize,
}

impl HeaderHandler {
    fn level(level: usize) -> Self {
        Self { level }
    }
}

impl ElementHandler for HeaderHandler {
    fn handle(&self, converter: &MarkdownConverter, node: &Handle, _attrs: &[html5ever::Attribute], output: &mut String, depth: usize) {
        output.push_str("\n");
        output.push_str(&"#".repeat(self.level));
        output.push(' ');
        converter.walk_children(node, output, depth);
        // output.push_str("\n");
    }
}

struct ParagraphHandler;
impl ElementHandler for ParagraphHandler {
    fn handle(&self, converter: &MarkdownConverter, node: &Handle, _attrs: &[html5ever::Attribute], output: &mut String, depth: usize) {
        output.push_str("\n");
        converter.walk_children(node, output, depth);
        output.push_str("\n");
    }
}

struct MarkHandler;
impl ElementHandler for MarkHandler {
    fn handle(&self, converter: &MarkdownConverter, node: &Handle, _attrs: &[html5ever::Attribute], output: &mut String, depth: usize) {
        output.push_str("==");
        converter.walk_children(node, output, depth);
        output.push_str("==");
    }
}

struct UnderlineHandler;
impl ElementHandler for UnderlineHandler {
    fn handle(&self, converter: &MarkdownConverter, node: &Handle, _attrs: &[html5ever::Attribute], output: &mut String, depth: usize) {
        output.push_str("<u>");
        converter.walk_children(node, output, depth);
        output.push_str("</u>");
    }
}

struct EmphasisHandler;
impl ElementHandler for EmphasisHandler {
    fn handle(&self, converter: &MarkdownConverter, node: &Handle, _attrs: &[html5ever::Attribute], output: &mut String, depth: usize) {
        output.push('*');
        converter.walk_children(node, output, depth);
        output.push('*');
    }
}

struct BoldHandler;
impl ElementHandler for BoldHandler {
    fn handle(&self, converter: &MarkdownConverter, node: &Handle, _attrs: &[html5ever::Attribute], output: &mut String, depth: usize) {
        output.push_str("**");
        converter.walk_children(node, output, depth);
        output.push_str("**");
    }
}

struct LinkHandler;
impl ElementHandler for LinkHandler {
    fn handle(&self, converter: &MarkdownConverter, node: &Handle, attrs: &[html5ever::Attribute], output: &mut String, depth: usize) {
        output.push('[');
        converter.walk_children(node, output, depth);
        output.push(']');
        output.push('(');
        
        if let Some(href) = attrs.iter().find(|attr| attr.name.local.as_ref() == "href") {
            output.push_str(&href.value);
        }
        
        output.push(')');
    }
}

struct SpanHandler;
impl ElementHandler for SpanHandler {
    fn handle(&self, converter: &MarkdownConverter, node: &Handle, attrs: &[html5ever::Attribute], output: &mut String, depth: usize) {
        let mut style_attr = None;
        for attr in attrs {
            if attr.name.local.as_ref() == "style" {
                style_attr = Some(attr.value.as_ref());
                break;
            }
        }

        if let Some(style_str) = style_attr {
            let styles = StyleParser::parse_styles(style_str);
            if let Some((prefix, suffix)) = StyleParser::get_markdown_style(&styles) {
                output.push_str(&prefix);
                converter.walk_children(node, output, depth);
                output.push_str(&suffix);
                return;
            }
        }

        // If no recognized styles, just process children
        converter.walk_children(node, output, depth);
    }
}

struct BlockquoteHandler;
impl ElementHandler for BlockquoteHandler {
    fn handle(&self, converter: &MarkdownConverter, node: &Handle, _attrs: &[html5ever::Attribute], output: &mut String, depth: usize) {
        // Add newline before blockquote if not at start
        if !output.is_empty() && !output.ends_with('\n') {
            output.push('\n');
        }

        // Create a temporary buffer for the quote content
        let mut quote_content = String::new();
        converter.walk_children(node, &mut quote_content, depth + 1);

        // Process the quote content line by line
        for line in quote_content.trim().lines() {
            output.push_str("> ");
            output.push_str(line);
            output.push('\n');
        }

        // Add extra newline after blockquote(optional)
        // output.push('\n');
    }
}

struct InlineCodeHandler;
impl ElementHandler for InlineCodeHandler {
    fn handle(&self, converter: &MarkdownConverter, node: &Handle, _attrs: &[html5ever::Attribute], output: &mut String, depth: usize) {
        output.push('`');
        converter.walk_children(node, output, depth);
        output.push('`');
    }
}

struct StrikeThroughHandler;
impl ElementHandler for StrikeThroughHandler {
    fn handle(&self, converter: &MarkdownConverter, node: &Handle, _attrs: &[html5ever::Attribute], output: &mut String, depth: usize) {
        output.push_str("~~");
        converter.walk_children(node, output, depth);
        output.push_str("~~");
    }
}

// For code blocks with pre + code
struct CodeBlockHandler;
impl ElementHandler for CodeBlockHandler {
    fn handle(&self, converter: &MarkdownConverter, node: &Handle, _attrs: &[html5ever::Attribute], output: &mut String, depth: usize) {
        // Get the code element which should be the child
        if let Some(code_node) = node.children.borrow().first() {
            if let NodeData::Element { attrs, .. } = &code_node.data {
                output.push_str("\n```");
                
                // Extract language if present
                for attr in attrs.borrow().iter() {
                    if attr.name.local.as_ref() == "class" {
                        if let Some(lang) = attr.value.as_ref().strip_prefix("language-") {
                            output.push_str(lang);
                            break;
                        }
                    }
                }
                output.push('\n');
                
                // Process the actual code content
                converter.walk_children(code_node, output, depth);
                output.push_str("\n```\n");
            }
        }
    }
}