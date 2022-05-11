//!
//! The Ethereal IR block element stack.
//!

pub mod element;

use self::element::Element;

///
/// The Ethereal IR block element stack.
///
#[derive(Debug, Default, Clone)]
pub struct Stack {
    /// The stack elements.
    pub elements: Vec<Element>,
}

impl Stack {
    /// The default stack size.
    pub const DEFAULT_STACK_SIZE: usize = 16;

    ///
    /// A shortcut constructor.
    ///
    pub fn new() -> Self {
        Self {
            elements: Vec::with_capacity(Self::DEFAULT_STACK_SIZE),
        }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn new_with_elements(elements: Vec<Element>) -> Self {
        Self { elements }
    }

    ///
    /// The stack state hash, which acts as a block identifier.
    ///
    /// Each block clone has its own initial stack state, which uniquely identifies the block.
    ///
    pub fn hash(&self) -> md5::Digest {
        let mut hash_context = md5::Context::new();
        for element in self.elements.iter() {
            match element {
                Element::Tag(tag) => hash_context.consume(tag.to_bytes_be()),
                _ => hash_context.consume([0]),
            }
        }
        hash_context.compute()
    }

    ///
    /// Pushes a stack element.
    ///
    pub fn push(&mut self, element: Element) {
        self.elements.push(element);
    }

    ///
    /// Pops a stack element.
    ///
    pub fn pop(&mut self) -> anyhow::Result<Element> {
        self.elements
            .pop()
            .ok_or_else(|| anyhow::anyhow!("Stack underflow"))
    }

    ///
    /// Pops the tag from the top.
    ///
    pub fn pop_tag(&mut self) -> anyhow::Result<num::BigUint> {
        match self.elements.pop() {
            Some(Element::Tag(tag)) => Ok(tag),
            Some(element) => anyhow::bail!("Expected tag, found {}", element),
            None => anyhow::bail!("Stack underflow"),
        }
    }

    ///
    /// Swaps two stack elements.
    ///
    pub fn swap(&mut self, index: usize) -> anyhow::Result<()> {
        if self.elements.len() < index + 1 {
            anyhow::bail!("Stack underflow");
        }

        let length = self.elements.len();
        self.elements.swap(length - 1, length - 1 - index);

        Ok(())
    }

    ///
    /// Duplicates a stack element.
    ///
    pub fn dup(&mut self, index: usize) -> anyhow::Result<()> {
        if self.elements.len() < index {
            anyhow::bail!("Stack underflow");
        }

        let dupped = self.elements[self.elements.len() - index].to_owned();
        self.elements.push(dupped);

        Ok(())
    }
}

impl std::fmt::Display for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[ {} ]",
            self.elements
                .iter()
                .map(Element::to_string)
                .collect::<Vec<String>>()
                .join(" | ")
        )
    }
}
