use crate::proto::{
    echo::{echo_server::Echo, EchoRequest, EchoResponse},
    hello::{hello_server::Hello, HelloRequest, HelloResponse},
};
use tonic::{async_trait, Request as TonicRequest, Response as TonicResponse, Status};

#[derive(Default)]
pub struct MyHello;

#[derive(Default)]
pub struct MyEcho;

#[async_trait]
impl Hello for MyHello {
    async fn say_hello(
        &self,
        request: TonicRequest<HelloRequest>,
    ) -> Result<TonicResponse<HelloResponse>, Status> {
        let reply = HelloResponse {
            message: format!("Hello From Tonic Server {}!", request.into_inner().name),
        };

        Ok(TonicResponse::new(reply))
    }
}

#[async_trait]
impl Echo for MyEcho {
    async fn ping(
        &self,
        request: TonicRequest<EchoRequest>,
    ) -> Result<TonicResponse<EchoResponse>, Status> {
        let reply = EchoResponse {
            message: request.into_inner().message,
        };

        Ok(TonicResponse::new(reply))
    }
}
