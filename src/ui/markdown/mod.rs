//! Markdown rendering support for AI responses.
//!
//! Focuses on proper text wrapping with structural elements (lists, headings, code blocks).

use crate::ui::theme::theme;
use gpui::{Div, SharedString, div, prelude::*};
use gpui_component::scroll::ScrollableElement;
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

/// Render markdown text as a GPUI Div element with proper text wrapping.
///
/// Supports:
/// - Numbered and unnumbered lists with proper wrapping
/// - Code blocks
/// - Headings
/// - Paragraphs with proper text wrapping
pub fn render_markdown(text: &str) -> Div {
    let t = theme();
    let parser = Parser::new(text);

    let mut container = div().w_full().flex().flex_col().content_stretch().gap_2();

    let mut current_text = String::new();
    let mut current_list_items = Vec::new();
    let mut list_item_number = 1;
    let mut is_ordered_list = false;
    let mut code_block_lines = Vec::new();
    let mut in_code_block = false;
    let mut in_list_item = false;
    let mut heading_level = HeadingLevel::H1;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {
                    current_text.clear();
                }
                Tag::List(start_num) => {
                    is_ordered_list = start_num.is_some();
                    list_item_number = start_num.unwrap_or(1) as usize;
                    current_list_items.clear();
                }
                Tag::Item => {
                    in_list_item = true;
                    current_text.clear();
                }
                Tag::Heading { level, .. } => {
                    heading_level = level;
                    current_text.clear();
                }
                Tag::CodeBlock(_) => {
                    in_code_block = true;
                    code_block_lines.clear();
                }
                _ => {}
            },
            Event::End(tag_end) => {
                match tag_end {
                    TagEnd::Paragraph => {
                        if !current_text.is_empty() && !in_list_item {
                            // Render standalone paragraph
                            let para = div()
                                .w_full()
                                .text_sm()
                                .text_color(t.item_title_color)
                                .line_height(t.markdown.paragraph_line_height)
                                .child(SharedString::from(current_text.trim().to_string()));

                            container = container.child(para);
                            current_text.clear();
                        }
                    }
                    TagEnd::List(_) => {
                        if !current_list_items.is_empty() {
                            let mut list_container = div().w_full().flex().flex_col().gap_1();

                            for item in current_list_items.drain(..) {
                                list_container = list_container.child(item);
                            }

                            container = container.child(list_container);
                        }
                        list_item_number = 1;
                    }
                    TagEnd::Item => {
                        in_list_item = false;
                        if !current_text.is_empty() {
                            let prefix = if is_ordered_list {
                                format!("{}. ", list_item_number)
                            } else {
                                "• ".to_string()
                            };

                            let item = div()
                                .w_full()
                                .flex()
                                .gap_2()
                                .child(
                                    div()
                                        .flex_shrink_0()
                                        .text_sm()
                                        .text_color(t.item_description_color)
                                        .child(SharedString::from(prefix)),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .text_sm()
                                        .text_color(t.item_title_color)
                                        .line_height(t.markdown.paragraph_line_height)
                                        .child(SharedString::from(current_text.trim().to_string())),
                                );

                            current_list_items.push(item);
                            current_text.clear();

                            if is_ordered_list {
                                list_item_number += 1;
                            }
                        }
                    }
                    TagEnd::Heading(_) => {
                        if !current_text.is_empty() {
                            let heading_div = match heading_level {
                                HeadingLevel::H1 => div().text_base(),
                                HeadingLevel::H2 => div().text_sm(),
                                _ => div().text_sm(),
                            };

                            let heading = heading_div
                                .w_full()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(t.item_title_color)
                                .mb_1()
                                .mt_1()
                                .line_height(t.markdown.heading_line_height)
                                .child(SharedString::from(current_text.trim().to_string()));

                            container = container.child(heading);
                            current_text.clear();
                        }
                    }
                    TagEnd::CodeBlock => {
                        if !code_block_lines.is_empty() {
                            // Join and trim to remove trailing empty lines
                            let code_text = code_block_lines.join("\n").trim_end().to_string();

                            let code_block = div()
                                .min_w_full()
                                .w_full()
                                .px_3()
                                .py_2()
                                .bg(t.item_background_selected)
                                .rounded(t.markdown.code_block_radius)
                                .overflow_x_scrollbar()
                                .font_family(t.markdown.code_font_family)
                                .text_xs()
                                .text_color(t.item_title_color)
                                .line_height(t.markdown.code_line_height)
                                .child(SharedString::from(code_text));

                            container = container.child(code_block);
                            code_block_lines.clear();
                        }
                        in_code_block = false;
                    }
                    _ => {}
                }
            }
            Event::Text(text) => {
                if in_code_block {
                    code_block_lines.push(text.to_string());
                } else {
                    current_text.push_str(&text);
                }
            }
            Event::Code(code) => {
                // Render inline code with markers for visibility
                current_text.push('`');
                current_text.push_str(&code);
                current_text.push('`');
            }
            Event::SoftBreak => {
                current_text.push(' ');
            }
            Event::HardBreak => {
                current_text.push('\n');
            }
            _ => {}
        }
    }

    // Flush any remaining text
    if !current_text.is_empty() {
        let para = div()
            .w_full()
            .text_sm()
            .text_color(t.item_title_color)
            .line_height(t.markdown.paragraph_line_height)
            .child(SharedString::from(current_text.trim().to_string()));
        container = container.child(para);
    }

    container
}
