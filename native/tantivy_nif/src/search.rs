extern crate tantivy;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, IndexWriter};
use tantivy::ReloadPolicy;

pub struct Searcher {
    schema: Schema,
    index: Index,
    index_writer: IndexWriter,
}

impl Searcher {
    pub fn new() -> Searcher {
        let mut schema_builder = SchemaBuilder::default();
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("body", TEXT);
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema.clone());
        let index_writer = index.writer(10_000_000).unwrap();

        Searcher {
            schema,
            index,
            index_writer,
        }
    }

    pub fn add_entry(&mut self, title: String, body: String) -> tantivy::Result<()> {
        let schema_title = self.schema.get_field("title").unwrap();
        let schema_body = self.schema.get_field("body").unwrap();
        self.index_writer.add_document(doc!(
            schema_title => title,
            schema_body => body,
        ));

        self.index_writer.commit()?;

        Ok(())
    }

    pub fn search(&self, query: String) -> tantivy::Result<Vec<Document>> {
        let reader = self.index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?;

        let searcher = reader.searcher();

        let query_parser = QueryParser::for_index(
            &self.index,
            vec![
                self.schema.get_field("title").unwrap(),
                self.schema.get_field("body").unwrap(),
            ],
        );

        let query = query_parser.parse_query(&query)?;
        let mut top_collector = TopDocs::with_limit(10);

        let top_docs = searcher.search(&*query, &mut top_collector)?;

        let docs = top_docs
            .iter()
            .map(|(_score, doc_address)| searcher.doc(*doc_address).unwrap())
            .collect();

        Ok(docs)
    }
}
