extern crate futures;
extern crate futures_cpupool;
extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate num_cpus;
extern crate reset_router;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_qs as qs;
extern crate tokio_core;

#[macro_use]
extern crate error_chain;

mod mandlebrot;
use mandlebrot::Mandlebrot;

use futures::future::Future;
use futures_cpupool::{Builder, CpuPool};

use hyper::header::{ContentLength, ContentType};
use hyper::server::Response;

use reset_router::hyper::{Context, Router};

lazy_static! {
    static ref POOL: CpuPool = {
        Builder::new().pool_size(num_cpus::get()).create()
    };
}

fn main() {
    use reset_router::hyper::ext::ServiceExtensions;

    let addr = "0.0.0.0:3000".parse().unwrap();

    let router = Router::build()
        .add_get(r"\A/\z", index)
        .add_not_found(not_found)
        .finish()
        .unwrap();

    router
        .quick_serve(num_cpus::get(), addr, || {
            ::tokio_core::reactor::Core::new().unwrap()
        })
        .unwrap();
}

type BoxFuture<I, E> = Box<Future<Item = I, Error = E>>;
type BoxedResponse = BoxFuture<hyper::Response, err::Error>;

pub mod err {
    error_chain! {
        errors { }

        foreign_links {
            Io(::std::io::Error);
            Hyper(::hyper::Error);
        }
    }

    impl ::reset_router::hyper::IntoResponse for Error {
        fn into_response(self) -> ::hyper::Response {
            use hyper::header::{ContentLength, ContentType};
            let msg = format!("{}", &self);
            ::hyper::Response::new()
                .with_status(::hyper::StatusCode::InternalServerError)
                .with_header(ContentLength(msg.len() as u64))
                .with_header(ContentType::plaintext())
                .with_body(msg)
        }
    }
}

fn not_found(_: Context) -> BoxedResponse {
    let msg = "NOT FOUND";
    Box::new(::futures::future::ok(
        Response::new()
            .with_status(::hyper::StatusCode::NotFound)
            .with_header(ContentLength(msg.len() as u64))
            .with_header(ContentType::plaintext())
            .with_body(msg),
    ))
}

fn index(ctx: Context) -> err::Result<Response> {
    #[derive(Clone, Debug, PartialEq, Deserialize)]
    #[serde(default)]
    struct Params {
        width: u32,
        height: u32,
    }

    impl Default for Params {
        fn default() -> Params {
            Params {
                width: 2048,
                height: 2048,
            }
        }
    }

    let params: Params;
    let req = ctx.into_request();
    match req.query() {
        Some(q) => {
            params = match qs::from_str(&q) {
                Ok(p) => p,
                Err(_) => Params::default(),
            }
        }
        None => params = Params::default(),
    };

    let mut img: Vec<u8> = Vec::new();
    let pool = POOL.clone();
    let mb = Mandlebrot::new_with_pool(pool);

    mb.generate(params.width, params.height, &mut img);

    let response = Response::new()
        .with_status(::hyper::StatusCode::Ok)
        .with_header(ContentLength(img.len() as u64))
        .with_header(ContentType::png())
        .with_body(img);
    Ok(response)
}
