use mnemosyne::routes;
use warp::Filter;
use handlebars::Handlebars;
use mnemosyne::templates::init_templates;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut registry = Handlebars::new();
    init_templates(&mut registry)?;

    let registry = Arc::new(registry);

    let handlebars = warp::any().map(move || registry.clone());

    let index = warp::path::end().and(handlebars).and_then(routes::index);

    let r#static = warp::path("static").and(warp::fs::dir("static"));

    warp::serve(index.or(r#static))
        .run(([0, 0, 0, 0], 8080))
        .await;

    Ok(())
}
