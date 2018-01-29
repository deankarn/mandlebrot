extern crate futures;
extern crate futures_cpupool;
extern crate hyper;

mod mandlebrot;
use mandlebrot::mandlebrot::Mandlebrot;

use futures::future::Future;
use futures_cpupool::{Builder, CpuPool};

use hyper::header::ContentType;
use hyper::server::{Http, Request, Response, Service};

const WIDTH: u32 = 2048;
const HEIGHT: u32 = 2048;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let env = Environment {
        pool: Builder::new().pool_size(8).create(),
    };
    let server = Http::new().bind(&addr, move || Ok(HttpService { env: &env }));
    server.unwrap().run().unwrap();
}

struct Environment {
    pool: CpuPool,
}

struct HttpService<'a> {
    pub env: &'a Environment,
}

impl<'a> Service for HttpService<'a> {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, _req: Request) -> Self::Future {
        let mut img: Vec<u8> = Vec::new();
        let pool = self.env.pool.clone();
        let mb = Mandlebrot::new_with_pool(pool);

        mb.generate(WIDTH, HEIGHT, &mut img);

        Box::new(futures::future::ok(
            Response::new()
                .with_header(ContentType::png())
                .with_body(img),
        ))
    }
}
