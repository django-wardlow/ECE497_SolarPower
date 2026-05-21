use core::cell::{Cell, RefCell};

use bytemuck::cast;
use embassy_net::Stack;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal, zerocopy_channel};
use embassy_time::Duration;
use esp_alloc as _;
use esp_println::println;
use picoserve::{AppBuilder, AppRouter, Router, response::{File, ws::{self, Message}}, routing::{self, PathRouter, get}};

use crate::psu_ctl::{PS_CMD, PsData};

pub const WEB_TASK_POOL_SIZE: usize = 1;

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
pub async fn web_task(task_id: usize, stack: Stack<'static>, rcv: zerocopy_channel::Receiver<'static, CriticalSectionRawMutex, PsData>) -> ! {
    let port = 80;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 2048];

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

    let router = picoserve::Router::new()
        .route("/",routing::get_service(File::html(include_str!("index.html"))))
        .route("/script.js", routing::get_service(File::javascript(include_str!("script.js"))))
        .route("/style.css", routing::get_service(File::css(include_str!("style.css"))))
        .route("/uPlot.iife.js", routing::get_service(File::javascript(include_str!("uPlot.iife.js"))))
        .route("/uPlot.min.css", routing::get_service(File::css(include_str!("uPlot.min.css"))))
        .route("/ws", get(async move |upg: ws::WebSocketUpgrade|{

            upg.on_upgrade_using_state(WShandler).with_protocol("messages")
        })).with_state(WSstate{rx: RefCell::new(rcv)});


    picoserve::Server::new(&router, config, &mut http_buffer)
        .listen_and_serve(task_id, stack, port, &mut tcp_rx_buffer, &mut tcp_tx_buffer)
        .await
        .into_never()
}

struct WSstate<'a>{
    rx: RefCell<zerocopy_channel::Receiver<'a, CriticalSectionRawMutex, PsData>>
}

struct WShandler;

impl ws::WebSocketCallbackWithState<WSstate<'_>> for WShandler{
    async fn run_with_state<R: picoserve::io::Read, W: picoserve::io::Write<Error = R::Error>>(
        self,
        state: &WSstate<'_>,
        mut rx: ws::SocketRx<R>,
        mut tx: ws::SocketTx<W>,
    ) -> Result<(), W::Error> {

        let mut message_buffer = [0; 128];

        let close_reason = loop {
            let mut ch = state.rx.borrow_mut();
            //println!("buf len is {}", ch.len());
            let buf = ch.receive();
            let message = match rx.next_message(&mut message_buffer, buf).await?
            {
                picoserve::futures::Either::First(Ok(message)) => message,
                picoserve::futures::Either::First(Err(error)) => {
                    break Some((error.code(), "Websocket Error"));
                }
                picoserve::futures::Either::Second(PsData) => {

                    let arr: [u8; 256] = cast(PsData.data);

                    //println!("sent ws message");

                    ch.receive_done();
                   
                    tx.send_binary(&arr).await?;
                    //tx.send_display(format_args!("{:?}", arr)).await?;

                    continue;
                    
                },
            };

            match message {
                Message::Text(new_message) => {
                    println!("ws msg: {}", new_message);
                    let mut parts = new_message.split(":");
                    match parts.next().unwrap() {
                        "V" => {
                            critical_section::with(|cs|{
                                PS_CMD.borrow_ref_mut(cs).v = parts.next().unwrap().parse().unwrap();
                            });
                        },
                        "I" => {
                            critical_section::with(|cs|{
                                PS_CMD.borrow_ref_mut(cs).i = parts.next().unwrap().parse().unwrap();
                            });
                        },
                        "F" => {},
                        "D" => {},
                        "M" => {
                            critical_section::with(|cs|{
                                PS_CMD.borrow_ref_mut(cs).mppt = (parts.next().unwrap().parse::<usize>().unwrap() == 1);
                            });
                        },
                        _ => ()
                    }
                }
                Message::Binary(message) => {
                }
                ws::Message::Close(reason) => {
                    break None;
                }
                Message::Ping(ping) => tx.send_pong(ping).await?,
                Message::Pong(_) => (),
            };
        };

        tx.close(close_reason).await
    
        
    }
}

