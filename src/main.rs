use anyhow::{Context, Result};
use handlebars::Handlebars;
use mdbook::book::{Book, Chapter};
use mdbook::config::BookConfig;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::BookItem;
use model::{FrontMatter, TemplateValues};
use serde_frontmatter::SerdeFMError;

mod model;

fn main() {
    mdbook_preprocessor_boilerplate::run(
        HandlebarsPreprocessor,
        "An mdbook preprocessor that does nothing", // CLI description
    );
}

struct HandlebarsPreprocessor;

impl Preprocessor for HandlebarsPreprocessor {
    fn name(&self) -> &str {
        "handlebars"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let templater = Templater::new(ctx)?;
        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                if let Err(err) = templater.template_chapter(chapter) {
                    eprintln!("{err}");
                }
            }
        });

        Ok(book)
    }
}

struct Templater<'a> {
    handlebars: Handlebars<'a>,
    book_config: &'a BookConfig,
}

impl<'a> Templater<'a> {
    pub fn new(ctx: &'a PreprocessorContext) -> Result<Self> {
        let config = ctx
            .config
            .get_preprocessor("handlebars")
            .context("Missing preprocessor.handlebars section")?;

        let template_folder = config
            .get("templates")
            .expect("Missing \"templates\" value in preprocessor.handlebars")
            .as_str()
            .context("\"templates\" should be a string")?;

        let template_folder = ctx.root.join(&ctx.config.book.src).join(template_folder);
        let mut handlebars = Handlebars::new();

        for (name, content) in template_folder
            .read_dir()?
            .filter_map(|el| el.ok())
            .map(|el| el.path())
            .filter(|el| el.extension().is_some_and(|ext| ext == "hbs"))
            .filter_map(|el| {
                if let (Some(name), Ok(content)) =
                    (el.file_stem(), std::fs::read_to_string(el.clone()))
                {
                    Some((name.to_str().unwrap().to_string(), content))
                } else {
                    eprintln!("Failed to register the template {}", el.display());
                    None
                }
            })
        {
            if let Err(err) = handlebars.register_template_string(&name, content) {
                eprintln!("Failed to parse {name}: {err}");
            }
        }

        Ok(Self {
            handlebars,
            book_config: &ctx.config.book,
        })
    }

    pub fn template_chapter(&self, chapter: &mut Chapter) -> Result<()> {
        match serde_frontmatter::deserialize(&chapter.content) {
            Ok((front_matter, content)) => {
                // Strip out front matter
                chapter.content = content;
                self.template(chapter, front_matter)?;
            }
            Err(SerdeFMError::MissingFrontMatter) => {}
            Err(SerdeFMError::YamlParseError(err)) => {
                return Err(err.into());
            }
        }
        Ok(())
    }

    fn template(&self, chapter: &mut Chapter, frontmatter: FrontMatter) -> Result<()> {
        if let Some(template) = frontmatter.template.clone() {
            let values = TemplateValues {
                book: self.book_config,
                chapter,
                frontmatter,
            };
            chapter.content = self.handlebars.render(&template, &values)?;
        }
        Ok(())
    }
}
