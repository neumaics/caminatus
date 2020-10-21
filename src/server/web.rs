use warp::ws::{WebSocket};
use warp::Filter;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tracing::{debug, info};
use futures::{FutureExt, StreamExt};

use crate::server::Command;

pub struct Web {
    pub sender: Sender<Command>,
    pub task_handle: tokio::task::JoinHandle<()>, // FIXME: This feels especially inelegant.
}

impl Web {
    pub fn start() -> Result<Self, String> {
        let (tx, mut rx) = mpsc::channel(16);
        
        let task_handle = tokio::spawn(async move {
            let ws = warp::path("ws")
                .and(warp::ws())
            // .and(manager)
                .map(|ws: warp::ws::Ws /*, manager: Manager*/| {
                    // ws.on_upgrade(move |socket| {
                    //     info!("somebody connected");                    
                    // })
                    ws.on_upgrade(|websocket| {
                        // Just echo all messages back...
                        let (tx, rx) = websocket.split();
                        rx.forward(tx).map(|result| {
                            if let Err(e) = result {
                                eprintln!("websocket error: {:?}", e);
                            }
                        })
                    })
                });

            let public = warp::path("public")
                .and(warp::fs::dir("public"));

            let routes = ws.or(public);

            warp::serve(routes)
                .run(([127, 0, 0, 1], 8080))
                .await;
        });

        Ok(Web {
            sender: tx,
            task_handle: task_handle
        })
    }
}

// async fn on_connect(self, ws: WebSocket) {
//     let (user_ws_tx, mut user_ws_rx) = ws.split();
//     let copy1 = self.clone();
//     let mut copy2 = self.clone();
//     let copy3 = self.clone();
//     let _monitor = copy1.monitor_service.receiver.clone();

//     info!("client connecting");

//     let (_tx, rx) = mpsc::unbounded_channel();
//     tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
//         debug!("{:?}", result);
//         if let Err(e) = result {
//             eprintln!("websocket send error: {}", e);
//         }
//     }));

//     // tokio::task::spawn(async move { 
//     //     while let Some(result) = monitor.recv().await {
//     //         let _ = tx.send(Ok(Message::text(result)));
//     //     }

//     //     copy1.on_disconnect()
//     // });

//     tokio::task::spawn(async move {
//         while let Some(result) = user_ws_rx.next().await {
//             debug!("{:?}", result);
//             let message = match result {
//                 Ok(message) => message,
//                 Err(error) => {
//                     debug!("{:?}", error);
//                     break;
//                 }
//             };
            
//             let command: Command = Command::from(message.to_str().unwrap());

//             let _ = copy2.sender.send(command).await;
//         }

//         let _ = copy3.on_disconnect();
//     });
// }

// async fn on_disconnect(self) {
//     info!("client diconnecting");
// }