use async_trait::async_trait;
use extio::Extio;
use http::{Request, Response};
use reqwest::Client;

#[derive(Debug)]
struct MyInterface;

#[derive(Debug)]
enum MyError {
    Reqwest,
}

#[async_trait]
impl Extio for MyInterface {
    type Error = MyError;

    async fn http_request(&self, req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, Self::Error> {
        let (parts, body) = req.into_parts();
        let uri = parts.uri.to_string();
        let client = Client::new();
        // start building reqwest request
        let mut builder = client.request(parts.method.clone(), &uri);

        // copy headers
        for (k, v) in parts.headers.iter() {
            builder = builder.header(k, v);
        }

        // add body
        builder = builder.body(body);

        // finalize into reqwest::Request
        let req = builder.build().map_err(|_| MyError::Reqwest)?;

        let response = client.execute(req).await.map_err(|err| {
            eprintln!("{err:?}");
            MyError::Reqwest
        })?;

        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response
            .bytes()
            .await
            .map_err(|_| MyError::Reqwest)?
            .to_vec();

        let mut builder = Response::builder().status(status);

        for (k, v) in headers.iter() {
            builder = builder.header(k, v);
        }

        builder.body(bytes).map_err(|_| MyError::Reqwest)
    }
}

#[tokio::main]
async fn main() {
    let interface = MyInterface;

    let req = Request::builder()
        .uri("https://www.rust-lang.org")
        .method("GET")
        .body(Vec::new())
        .unwrap();
    let respon = interface.http_request(req).await.unwrap();

    println!("{:?}", String::from_utf8(respon.body().to_vec()));
}
