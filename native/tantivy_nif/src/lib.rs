#[macro_use]
extern crate tantivy;
#[macro_use]
extern crate rustler;

use rustler::resource::ResourceArc;
use rustler::{Encoder, Env, Error, NifResult, Term};
use search::Searcher;
use std::sync::Mutex;

struct SearcherResource(Mutex<Searcher>);

mod atoms;
mod search;

rustler_export_nifs! {
    "Elixir.Tantivy.NIF",
    [
        ("init", 0, init),
        ("search", 2, search),
        ("add_entry", 3, add_entry)
    ],
    Some(load)
}

fn load(env: Env, _info: Term) -> bool {
    resource_struct_init!(SearcherResource, env);
    true
}

fn init<'a>(env: Env<'a>, _args: &[Term<'a>]) -> NifResult<Term<'a>> {
    let resource = ResourceArc::new(SearcherResource(Mutex::new(Searcher::new())));
    let resp = (atoms::ok(), resource);
    Ok(resp.encode(env))
}

fn search<'a>(env: Env<'a>, args: &[Term<'a>]) -> NifResult<Term<'a>> {
    let resource: ResourceArc<SearcherResource> = args[0].decode()?;
    let query: String = args[1].decode()?;

    let searcher = match resource.0.try_lock() {
        Ok(guard) => guard,
        Err(_) => return Err(Error::BadArg),
    };

    match searcher.search(query) {
        Ok(docs) => {
            let terms: Vec<Term<'a>> = docs.into_iter().map(|doc| doc_to_term(env, doc)).collect();
            Ok(terms.encode(env))
        }
        Err(_) => Ok(atoms::error().encode(env)),
    }
}

fn add_entry<'a>(env: Env<'a>, args: &[Term<'a>]) -> NifResult<Term<'a>> {
    let resource: ResourceArc<SearcherResource> = args[0].decode()?;
    let title: String = args[1].decode()?;
    let body: String = args[2].decode()?;

    let mut searcher = match resource.0.try_lock() {
        Ok(guard) => guard,
        Err(_) => return Err(Error::BadArg),
    };

    match searcher.add_entry(title, body) {
        Ok(_) => Ok(atoms::ok().encode(env)),
        Err(_) => Ok(atoms::error().encode(env)),
    }
}

fn doc_to_term<'a>(env: Env<'a>, doc: tantivy::Document) -> Term<'a> {
    let terms: Vec<Term<'a>> = doc
        .field_values()
        .into_iter()
        .map(|fv| match fv.value() {
            tantivy::schema::Value::Str(v) => v.encode(env),
            tantivy::schema::Value::U64(v) => v.encode(env),
            tantivy::schema::Value::I64(v) => v.encode(env),
            tantivy::schema::Value::F64(v) => v.encode(env),
            tantivy::schema::Value::Bytes(v) => v.encode(env),
            _ => atoms::error().encode(env),

        }).collect();
    terms.encode(env)
}
