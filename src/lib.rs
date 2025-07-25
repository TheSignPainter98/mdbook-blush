use anyhow::Result;
use mdbook::book::{Book, Chapter};
use mdbook::errors::Result as MdbookResult;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::BookItem;
use pulldown_cmark::{Event, Options, Parser};

pub struct BlushPreprocessor;

impl BlushPreprocessor {
    pub fn new() -> Self {
        Self
    }

    fn preprocess_bookitem(&self, item: &mut BookItem) -> Result<()> {
        match item {
            BookItem::Chapter(chapter) => self.preprocess_chapter(chapter),
            BookItem::Separator | BookItem::PartTitle(_) => Ok(()),
        }
    }

    fn preprocess_chapter(&self, chapter: &mut Chapter) -> Result<()> {
        let parser = Parser::new_ext(&chapter.content, Options::all()).map(|event| match event {
            Event::Text(text) => Event::Text(self.preprocess_text(&text).into()),
            _ => event,
        });
        let new_content_capacity = (chapter.content.len() as f64 * 1.05) as usize;
        let mut new_content = String::with_capacity(new_content_capacity);
        pulldown_cmark_to_cmark::cmark(parser, &mut new_content)?;
        chapter.content = new_content;

        for sub_item in &mut chapter.sub_items {
            self.preprocess_bookitem(sub_item)?;
        }

        Ok(())
    }

    fn preprocess_text(&self, text: &str) -> String {
        let mut fragments = Vec::with_capacity(8);
        let mut cursor = 0;
        let mut char_indices = text
            .char_indices()
            .filter(|(_, chr)| *chr == '=' || *chr == ' ' || *chr == '\t');
        while let Some((index, chr)) = char_indices.next() {
            if chr != '=' {
                continue;
            }

            fragments.push(&text[cursor..index]);
            cursor = index;

            if let Some((end_index, chr)) = char_indices.next() {
                if chr != '=' || end_index == 1 + index {
                    fragments.push(&text[cursor..end_index]);
                    cursor = end_index;
                    continue;
                }

                fragments.push(r#"<span class="small-caps">"#);
                fragments.push(&text[(cursor + 1)..end_index]);
                fragments.push(r#"</span>"#);
                cursor = end_index + 1;
            }
        }

        fragments.push(&text[cursor..]);
        fragments.concat()
    }
}

impl Preprocessor for BlushPreprocessor {
    fn name(&self) -> &str {
        "mdbook-blush"
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> MdbookResult<Book> {
        book.sections
            .iter_mut()
            .try_for_each(|section| self.preprocess_bookitem(section))?;
        Ok(book)
    }
}
