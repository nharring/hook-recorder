use super::templating::TemplateSingleton;
use super::{display, healthcheck, record};
use diesel::{pg::PgConnection, r2d2::ConnectionManager};
use log::debug;
use warp::Filter;

pub fn gen_filters(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
    templater: TemplateSingleton,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + 'static {
    debug!("Beginning filter intialization");
    gen_display(pool.clone(), templater.clone())
        .or(gen_record(pool.clone()))
        .or(gen_healthcheck(pool.clone(), templater.clone()))
}

fn gen_display(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>, templater: TemplateSingleton,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + 'static {
    debug!("Initializing display filter");
    warp::path!("display")
        .and(warp::get())
        .and(with_db(pool))
        .and(with_templater(templater))
        .and_then(display::display_last)
}

fn gen_record(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + 'static {
    debug!("Initializing record filter");
    warp::path!("record")
        .and(warp::post())
        .and(warp::body::bytes())
        .and(warp::header::headers_cloned())
        .and(with_db(pool))
        .and_then(|body, headers, pool| record::record_webhook(pool, body, headers))
}

fn gen_healthcheck(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>, templater: TemplateSingleton,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone + 'static {
    debug!("Initializing healthcheck filter");
    warp::path!("healthcheck")
        .and(warp::get())
        .and(with_db(pool))
        .and(with_templater(templater))
        .and_then(healthcheck::healthcheck)
}

fn with_db(
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
) -> impl Filter<
    Extract = (r2d2::Pool<ConnectionManager<PgConnection>>,),
    Error = std::convert::Infallible,
> + Clone
       + 'static {
    warp::any().map(move || pool.clone())
}

fn with_templater(
    templater: TemplateSingleton,
) -> impl Filter<Extract = (TemplateSingleton,), Error = std::convert::Infallible> + Clone + 'static
{
    warp::any().map(move || templater.clone())
}
