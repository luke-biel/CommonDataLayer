use handlebars::Handlebars;
use std::sync::Arc;
use serde_json::json;

pub async fn index(handlebars: Arc<Handlebars<'_>>) -> Result<impl warp::Reply, warp::Rejection> {
    let html = handlebars.render("layout", &json!({})).map_err(|_e| warp::reject())?;

    Ok(warp::reply::html(html))
}
