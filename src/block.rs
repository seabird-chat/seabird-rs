use crate::proto;

/// A builder for creating message blocks.
///
/// # Examples
///
/// ```rust
/// use seabird::Block;
///
/// // Simple text with formatting
/// let block = Block::new()
///     .text("Hello ")
///     .bold("world")
///     .text("!");
///
/// // Nested formatting
/// let block = Block::new()
///     .text("This is ")
///     .bold(Block::new().italic("very").text(" important"));
///
/// // Lists
/// let block = Block::new()
///     .text("My list:")
///     .list(vec!["Item 1", "Item 2", "Item 3"]);
///
/// // Combining blocks with append/prepend
/// let header = Block::new().heading(1, "Title");
/// let body = Block::new().text("Content");
/// let block = Block::new()
///     .append(header)
///     .append(body);
/// ```
#[derive(Clone, Debug)]
pub struct Block {
    children: Vec<proto::Block>,
}

impl Block {
    /// Creates a new empty block builder.
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// Appends blocks from another Block or proto::Block to the end.
    pub fn append(mut self, block: impl Into<Block>) -> Self {
        let block: Block = block.into();
        self.children.extend(block.children);
        self
    }

    /// Prepends blocks from another Block or proto::Block to the beginning.
    pub fn prepend(mut self, block: impl Into<Block>) -> Self {
        let block: Block = block.into();
        let mut new_children = block.children;
        new_children.extend(self.children);
        self.children = new_children;
        self
    }

    /// Adds a text block to the sequence.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        self.children.push(proto::Block {
            plain: String::new(),
            inner: Some(proto::block::Inner::Text(proto::TextBlock { text })),
        });
        self
    }

    /// Adds a bold-formatted block.
    pub fn bold(mut self, content: impl Into<Block>) -> Self {
        let inner_block = content.into().into();
        self.children.push(proto::Block {
            plain: String::new(),
            inner: Some(proto::block::Inner::Bold(Box::new(proto::BoldBlock {
                inner: Some(Box::new(inner_block)),
            }))),
        });
        self
    }

    /// Adds an italic-formatted block.
    pub fn italic(mut self, content: impl Into<Block>) -> Self {
        let inner_block = content.into().into();
        self.children.push(proto::Block {
            plain: String::new(),
            inner: Some(proto::block::Inner::Italics(Box::new(proto::ItalicsBlock {
                inner: Some(Box::new(inner_block)),
            }))),
        });
        self
    }

    /// Adds an underline-formatted block.
    pub fn underline(mut self, content: impl Into<Block>) -> Self {
        let inner_block = content.into().into();
        self.children.push(proto::Block {
            plain: String::new(),
            inner: Some(proto::block::Inner::Underline(Box::new(proto::UnderlineBlock {
                inner: Some(Box::new(inner_block)),
            }))),
        });
        self
    }

    /// Adds a strikethrough-formatted block.
    pub fn strikethrough(mut self, content: impl Into<Block>) -> Self {
        let inner_block = content.into().into();
        self.children.push(proto::Block {
            plain: String::new(),
            inner: Some(proto::block::Inner::Strikethrough(Box::new(proto::StrikethroughBlock {
                inner: Some(Box::new(inner_block)),
            }))),
        });
        self
    }

    /// Adds a spoiler-formatted block.
    pub fn spoiler(mut self, content: impl Into<Block>) -> Self {
        let inner_block = content.into().into();
        self.children.push(proto::Block {
            plain: String::new(),
            inner: Some(proto::block::Inner::Spoiler(Box::new(proto::SpoilerBlock {
                inner: Some(Box::new(inner_block)),
            }))),
        });
        self
    }

    /// Adds a blockquote-formatted block.
    pub fn blockquote(mut self, content: impl Into<Block>) -> Self {
        let inner_block = content.into().into();
        self.children.push(proto::Block {
            plain: String::new(),
            inner: Some(proto::block::Inner::Blockquote(Box::new(proto::BlockquoteBlock {
                inner: Some(Box::new(inner_block)),
            }))),
        });
        self
    }

    /// Adds an inline code block.
    pub fn inline_code(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        self.children.push(proto::Block {
            plain: String::new(),
            inner: Some(proto::block::Inner::InlineCode(proto::InlineCodeBlock {
                text,
            })),
        });
        self
    }

    /// Adds a fenced code block with optional language info.
    pub fn fenced_code(mut self, info: impl Into<String>, text: impl Into<String>) -> Self {
        let info = info.into();
        let text = text.into();
        self.children.push(proto::Block {
            plain: String::new(),
            inner: Some(proto::block::Inner::FencedCode(proto::FencedCodeBlock {
                info,
                text,
            })),
        });
        self
    }

    /// Adds a link block with a URL and content.
    pub fn link(mut self, url: impl Into<String>, content: impl Into<Block>) -> Self {
        let url = url.into();
        let inner_block = content.into().into();
        self.children.push(proto::Block {
            plain: String::new(),
            inner: Some(proto::block::Inner::Link(Box::new(proto::LinkBlock {
                url,
                inner: Some(Box::new(inner_block)),
            }))),
        });
        self
    }

    /// Adds a heading block with a level and content.
    pub fn heading(mut self, level: i32, content: impl Into<Block>) -> Self {
        let inner_block = content.into().into();
        self.children.push(proto::Block {
            plain: String::new(),
            inner: Some(proto::block::Inner::Heading(Box::new(proto::HeadingBlock {
                level,
                inner: Some(Box::new(inner_block)),
            }))),
        });
        self
    }

    /// Adds a list block containing multiple items.
    pub fn list(mut self, items: impl IntoIterator<Item = impl Into<Block>>) -> Self {
        let inner: Vec<proto::Block> = items.into_iter().map(|item| item.into().into()).collect();
        self.children.push(proto::Block {
            plain: String::new(),
            inner: Some(proto::block::Inner::List(proto::ListBlock { inner })),
        });
        self
    }

    /// Adds a timestamp block.
    pub fn timestamp(mut self, time: std::time::SystemTime) -> Self {
        let duration = time
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let timestamp = prost_types::Timestamp {
            seconds: duration.as_secs() as i64,
            nanos: duration.subsec_nanos() as i32,
        };
        self.children.push(proto::Block {
            plain: String::new(),
            inner: Some(proto::block::Inner::Timestamp(proto::TimestampBlock {
                inner: Some(timestamp),
            })),
        });
        self
    }

    /// Adds a container block with multiple child blocks.
    pub fn container(mut self, blocks: impl IntoIterator<Item = impl Into<Block>>) -> Self {
        let inner: Vec<proto::Block> = blocks.into_iter().map(|b| b.into().into()).collect();
        self.children.push(proto::Block {
            plain: String::new(),
            inner: Some(proto::block::Inner::Container(proto::ContainerBlock {
                inner,
            })),
        });
        self
    }
}

impl From<Block> for proto::Block {
    fn from(block: Block) -> Self {
        // If single child, return it directly
        if block.children.len() == 1 {
            return block.children.into_iter().next().unwrap();
        }

        // Otherwise wrap in ContainerBlock
        proto::Block {
            plain: String::new(),
            inner: Some(proto::block::Inner::Container(proto::ContainerBlock {
                inner: block.children,
            })),
        }
    }
}

impl From<proto::Block> for Block {
    fn from(block: proto::Block) -> Self {
        // If it's a ContainerBlock, unwrap its children
        if let Some(proto::block::Inner::Container(container)) = block.inner {
            Block {
                children: container.inner,
            }
        } else {
            // Otherwise, wrap the single block
            Block {
                children: vec![block],
            }
        }
    }
}

impl From<&str> for Block {
    fn from(text: &str) -> Self {
        Block::new().text(text)
    }
}

impl From<String> for Block {
    fn from(text: String) -> Self {
        Block::new().text(text)
    }
}

impl From<Block> for crate::client::MessageContent {
    fn from(block: Block) -> Self {
        crate::client::MessageContent::Blocks(block.into())
    }
}
