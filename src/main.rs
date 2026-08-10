use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use axum::{Router, routing::get};
use hyper::{
    Request, Response, body::Incoming, client::conn::http1, server::conn::http1::Builder,
    service::service_fn,
};
use hyper_util::rt::TokioIo;
use tokio::{
    net::{TcpListener, TcpStream}
};

const BACKEND_1_ADDR: &str = "127.0.0.1:8081";
const BACKEND_2_ADDR: &str = "127.0.0.1:8082";
const LISTENER_ADDR: &str = "0.0.0.0:3000";

struct BackendServer {
    addr: String,
    num: u8,
}

impl BackendServer {
    async fn new(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let num = Arc::new(self.num);
        let router: Router = Router::new().route(
            "/",
            get(|| async move { format!("Welcome to Backend Server {}", num) }),
        );

        let listener = TcpListener::bind(&self.addr).await?;
        println!("{:?}", listener);
        axum::serve(listener, router).await?;
        Ok(())
    }
}

struct LoadBalancer {
    backends: Vec<SocketAddr>,
    counter: AtomicUsize,
}

impl LoadBalancer {
    fn next_backend(&self) -> SocketAddr {
        let idx = self.counter.fetch_add(1, Ordering::Relaxed);
        self.backends[idx % self.backends.len()]
    }
}

async fn forward_request(
    lb: Arc<LoadBalancer>,
    req: Request<Incoming>,
) -> Result<Response<Incoming>, hyper::Error> {
    let target_addr = lb.next_backend();

    let backend_stream = TcpStream::connect(target_addr).await.unwrap();
    let io = TokioIo::new(backend_stream);

    let (mut sender, conn) = http1::handshake(io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            eprintln!("Backend connection Error {}", err);
        }
    });

    sender.send_request(req).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let servers: Arc<Vec<SocketAddr>> = Arc::new(vec![ BACKEND_1_ADDR.parse()?, BACKEND_2_ADDR.parse()?]);

    let _backend_1 = tokio::spawn(async move {
        if let Ok(_) = BackendServer::new(&BackendServer {
            addr: BACKEND_1_ADDR.to_string(),
            num: 1,
        })
        .await
        {
            println!("Backend Server 1 Running");
        }
    });
    let _backend_2 = tokio::spawn(async move {
        if let Ok(_) = BackendServer::new(&BackendServer {
            addr: BACKEND_2_ADDR.to_string(),
            num: 2,
        })
        .await
        {
            println!("Backend Server 2 Running");
        }
    });
    let backends = Arc::new(LoadBalancer {
        backends: servers.to_vec(),
        counter: AtomicUsize::new(0),
    });

    let listener = TcpListener::bind(LISTENER_ADDR).await?;
    println!("Load Balancer Started {:?}", listener);
    let _health_check = tokio::spawn(async move {});

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let lb_clone = Arc::clone(&backends);

        tokio::spawn(async move {
            let service = service_fn(move |req| {
                forward_request(Arc::clone(&lb_clone), req)
            });

            if let Err(e) = Builder::new().serve_connection(io, service).await {
                eprintln!("Error serving connection: {:?}", e);
            }
        });
    }
}
