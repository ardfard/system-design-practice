use tonic::{transport::Server, Request, Response, Status};
use url::Url;

use urlshortenergrpc::url_shortener_server::{UrlShortener, UrlShortenerServer};
use urlshortenergrpc::{GetRealUrlRequest, GetRealUrlResponse, ShortenRequest, ShortenResponse};

use hyper::service::{make_service_fn, service_fn};

use hyper;

mod urlshortenergrpc {
    tonic::include_proto!("urlshortener");
}

pub struct UrlShortenerLive {
    app: Box<dyn url_shortener::UrlShortener + Sync + Send>,
}

#[tonic::async_trait]
impl UrlShortener for UrlShortenerLive {
    async fn shorten(
        &self,
        request: Request<ShortenRequest>,
    ) -> Result<Response<ShortenResponse>, Status> {
        let msg = request.get_ref();

        let url = Url::parse(msg.url.as_str())
            .map_err(|_| tonic::Status::invalid_argument("Url is invalid!"))?;

        let result = self.app.shorten(url).await;

        match result {
            Ok(short_url) => Ok(Response::new(ShortenResponse {
                shortened_url: short_url,
            })),
            Err(_) => Err(tonic::Status::internal("Server is not available!")),
        }
    }

    async fn get_real_url(
        &self,
        request: Request<GetRealUrlRequest>,
    ) -> Result<Response<GetRealUrlResponse>, Status> {
        let msg = request.get_ref();

        let result = self.app.get_real_url(msg.id).await;

        // Convert the result to response
        match result {
            Ok(url) => Ok(Response::new(GetRealUrlResponse { url })),
            Err(e) => Err(tonic::Status::internal(format!(
                "Server is not available! {:?}",
                e
            ))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let shortener = url_shortener::IncrementalUrlShortener::new("sqlite:test.db").await?;
    let urlshortener = UrlShortenerLive {
        app: Box::new(shortener),
    };

    println!("Start listening {:?}", addr);

    let grpc_task = tokio::spawn(async move {
        Server::builder()
            .add_service(UrlShortenerServer::new(urlshortener))
            .serve(addr)
            .await
            .map_err(|e| eprintln!("Server error: {}", e));
    });

    let http_server = make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(service_fn(|_req| async {
            Ok::<_, hyper::Error>(hyper::Response::new(hyper::Body::from("Hello World!")))
        }))
    });

    let http_addr = "0.0.0.0:50052".parse()?;
    let http_task = tokio::spawn(async move {
        println!("Start listening {:?}", http_addr);
        hyper::Server::bind(&http_addr)
            .serve(http_server)
            .await
            .map_err(|e| {
                eprintln!("Server error: {}", e);
            })
    });

    let (_, _) = tokio::join!(grpc_task, http_task);

    let future = async { 2 };

    future.map(|x| x + 1).await;

    Ok(())
}
