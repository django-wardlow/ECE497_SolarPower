use embassy_net::Stack;
use embassy_sync::signal::Signal;
use embassy_time::Duration;
use esp_alloc as _;
use picoserve::{AppBuilder, AppRouter, Router, response::{File, ws}, routing::{self, get}};

pub struct Application;

impl AppBuilder for Application {
    type PathRouter = impl routing::PathRouter;

    fn build_app(self) -> picoserve::Router<Self::PathRouter> {
        picoserve::Router::new()
        .route("/",routing::get_service(File::html(include_str!("index.html"))))
        .route("/script.js", routing::get_service(File::javascript(include_str!("script.js"))))
        .route("/style.css", routing::get_service(File::css(include_str!("style.css"))))
        .route("/uPlot.iife.js", routing::get_service(File::javascript(include_str!("uPlot.iife.js"))))
        .route("/uPlot.min.css", routing::get_service(File::css(include_str!("uPlot.min.css"))))
        // .route("/ws", get(async move |upg: ws::WebSocketUpgrade|{

        //     upg.on_upgrade_using_state(callback)
        // }))
    }
}

pub const WEB_TASK_POOL_SIZE: usize = 2;

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
pub async fn web_task(
    task_id: usize,
    stack: Stack<'static>,
    router: &'static AppRouter<Application>,
    config: &'static picoserve::Config<Duration>,
) -> ! {
    let port = 80;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

    picoserve::Server::new(router, config, &mut http_buffer)
        .listen_and_serve(task_id, stack, port, &mut tcp_rx_buffer, &mut tcp_tx_buffer)
        .await
        .into_never()
}

// struct WSstate{

// }

// struct WShandler;

// impl ws::WebSocketCallbackWithState<WSstate> for WShandler{
//     async fn run_with_state<R: picoserve::io::Read, W: picoserve::io::Write<Error = R::Error>>(
//         self,
//         state: &WSstate,
//         rx: ws::SocketRx<R>,
//         tx: ws::SocketTx<W>,
//     ) -> Result<(), W::Error> {

//         let mut message_buffer = [0; 128];

//         let close_reason = loop {
//             let message = match rx.next_message(&mut message_buffer, Signal::).await?
//             {
//                 picoserve::futures::Either::First(Ok(message)) => message,
//                 picoserve::futures::Either::First(Err(error)) => {
//                     break Some((error.code(), "Websocket Error"));
//                 }
//                 picoserve::futures::Either::Second(message_changed) => match message_changed {
//                     Ok(message) => {
//                         tx.send_display(format_args!("Message: {message}")).await?;
//                         continue;
//                     }
//                     Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
//                         tx.send_display(format_args!("Missed {n} messages")).await?;
//                         continue;
//                     }
//                     Err(tokio::sync::broadcast::error::RecvError::Closed) => {
//                         break Some((1011, "Server has an error"));
//                     }
//                 },
//             };

//             log::info!("Message: {message:?}");
//             match message {
//                 Message::Text(new_message) => {
//                     let _ = messages_tx.send(new_message.into());
//                 }
//                 Message::Binary(message) => {
//                     log::info!("Ignoring binary message: {message:?}")
//                 }
//                 ws::Message::Close(reason) => {
//                     log::info!("Websocket close reason: {reason:?}");
//                     break None;
//                 }
//                 Message::Ping(ping) => tx.send_pong(ping).await?,
//                 Message::Pong(_) => (),
//             };
//         };

//         tx.close(close_reason).await
    
        
//     }
// }


pub struct WebApp {
    pub router: &'static Router<<Application as AppBuilder>::PathRouter>,
    pub config: &'static picoserve::Config<Duration>,
}

impl Default for WebApp {
    fn default() -> Self {
        let router = picoserve::make_static!(AppRouter<Application>, Application.build_app());

        let config = picoserve::make_static!(
            picoserve::Config<Duration>,
            picoserve::Config::new(picoserve::Timeouts {
                start_read_request: Some(Duration::from_secs(5)),
                read_request: Some(Duration::from_secs(1)),
                write: Some(Duration::from_secs(1)),
                persistent_start_read_request: Some(Duration::from_secs(1)),
            })
            .keep_connection_alive()
        );

        Self { router, config }
    }
}
