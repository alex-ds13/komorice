use crate::Message;

use std::io::BufReader;
use std::io::Read;
use std::time::Duration;

use iced::futures::SinkExt;
use iced::Subscription;
use komorebi_client::SocketMessage;
use komorebi_client::SubscribeOptions;

// enum State {
//     Starting,
//     Ready(UnixListener),
// }

pub fn connect() -> Subscription<Message> {
    struct Worker;
    let id = std::any::TypeId::of::<Worker>();

    Subscription::run_with_id(
        id,
        iced::stream::channel(10, move |mut output| async move {
            // let mut state = State::Starting;
            let subscriber_name = "komofig";

            // loop {
                // match state {
                    // State::Starting => {
                        // let (sender, receiver) = channel::bounded(10);

                        // let sender_clone = sender.clone();
                        // match output.send(Message::OptionsFileWatcherTx(sender_clone)).await {
                        //     Ok(_) => {},
                        //     Err(e) => {
                        //         println!("Error trying to send the options watcher sender:\n{e:?}");
                        //     },
                        // }

                        let listener = komorebi_client::subscribe_with_options(subscriber_name, SubscribeOptions {
                            filter_state_changes: true,
                        })
                        .expect("could not subscribe to komorebi notifications");

                        println!("subscribed to komorebi notifications: \"{}\"", subscriber_name);

                        // state = State::Ready(listener);
                    // }
                    // State::Ready(listener) => {
                    //     let mut output_c = output.clone();
                        // let listener_c = listener.try_clone().expect("couldn't clone socket");
                        async_std::task::spawn_blocking(move || {
                            for client in listener.incoming() {
                                match client {
                                    Ok(subscription) => {
                                        let mut buffer = Vec::new();
                                        let mut reader = BufReader::new(subscription);

                                        // this is when we know a shutdown has been sent
                                        if matches!(reader.read_to_end(&mut buffer), Ok(0)) {
                                            println!("disconnected from komorebi");

                                            // keep trying to reconnect to komorebi
                                            while komorebi_client::send_message(
                                                &SocketMessage::AddSubscriberSocket(subscriber_name.to_string()),
                                            )
                                                .is_err()
                                                {
                                                    async_std::task::block_on(async {
                                                        async_std::task::yield_now().await;
                                                        async_std::task::sleep(Duration::from_secs(1)).await;
                                                    });
                                                }

                                            println!("reconnected to komorebi");
                                        }

                                        match String::from_utf8(buffer) {
                                            Ok(notification_string) => {
                                                match serde_json::from_str::<komorebi_client::Notification>(
                                                    &notification_string,
                                                ) {
                                                    Ok(notification) => {
                                                        println!("received notification from komorebi");

                                                        async_std::task::block_on(async {
                                                            if let Err(error) = output.send(Message::KomorebiNotification(notification)).await {
                                                                println!("could not send komorebi notification update to gui thread: {error}")
                                                            }
                                                        });

                                                        // ctx_komorebi.request_repaint();
                                                    }
                                                    Err(error) => {
                                                        println!("could not deserialize komorebi notification: {error}");
                                                    }
                                                }
                                            }
                                            Err(error) => {
                                                println!(
                                                    "komorebi notification string was invalid utf8: {error}"
                                                )
                                            }
                                        }
                                    }
                                    Err(error) => {
                                        println!("{error}");
                                    }
                                }
                            }
                            // listener
                        }).await;
                        
                    // }
                // }
            // }
        }),
    )
}
