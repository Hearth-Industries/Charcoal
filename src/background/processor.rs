use crate::background::connector::send_message;
use crate::CharcoalConfig;
use hearth_interconnect::errors::ErrorReport;
use hearth_interconnect::messages::{Message, Metadata};
use log::{debug, error};
use rdkafka::consumer::BaseConsumer;
use rdkafka::producer::FutureProducer;
use rdkafka::Message as KafkaMessage;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use prokio::pinned::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Clone, Debug)]
pub struct FromBackgroundData {
    pub message: Message,
}

#[derive(Clone, Debug)]
pub struct FromMainData {
    pub message: Message,
    pub response_tx: Arc<UnboundedSender<IPCData>>,
    pub guild_id: String,
}

#[derive(Clone, Debug)]
pub enum IPCData {
    FromBackground(FromBackgroundData),
    FromMain(FromMainData),
    ErrorReport(ErrorReport),
    MetadataResult(Metadata),
}

// Makes things slightly easier
impl IPCData {
    pub fn new_from_main(
        message: Message,
        sender: Arc<UnboundedSender<IPCData>>,
        guild_id: String,
    ) -> IPCData {
        IPCData::FromMain(FromMainData {
            message,
            response_tx: sender,
            guild_id,
        })
    }
    pub fn new_from_background(message: Message) -> IPCData {
        IPCData::FromBackground(FromBackgroundData { message })
    }
}

pub async fn parse_message(
    message: Message,
    guild_id_to_tx: &mut HashMap<String, Arc<UnboundedSender<IPCData>>>,
    from_bg_tx: &mut UnboundedSender<IPCData>,
) {
    match &message {
        Message::ErrorReport(e) => {
            error!("GOT Error: {:?} From Hearth Server", e);
            let tx = guild_id_to_tx.get_mut(&e.guild_id);
            match tx {
                Some(tx) => {
                    let gt = tx.send_now(IPCData::ErrorReport(e.clone()));
                    match gt {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Failed to send error report with error: {:?}", e);
                        }
                    }
                }
                None => {
                    error!("Failed to get appropriate sender when attempting to send error report")
                }
            }
        }
        Message::ExternalJobExpired(_je) => {
            let r = from_bg_tx.send_now(IPCData::new_from_background(message));
            match r {
                Ok(_) => {}
                Err(e) => {
                    error!(
                        "Failed to send Kafka message to main thread once received with error: {}!",
                        e
                    )
                }
            }
        }
        Message::WorkerShutdownAlert(_) => {
            let r = from_bg_tx.send_now(IPCData::new_from_background(message));
            match r {
                Ok(_) => {}
                Err(e) => {
                    error!(
                        "Failed to send Kafka message to main thread once received with error: {}!",
                        e
                    )
                }
            }
        }
        Message::ExternalQueueJobResponse(r) => {
            let tx = guild_id_to_tx.get_mut(&r.guild_id);
            match tx {
                Some(tx) => {
                    let r = tx.send_now(IPCData::new_from_background(message));
                    match r {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Failed to send Kafka message to main thread once received with error: {}!",e)
                        }
                    }
                }
                None => {
                    error!("Failed to send Response from BG Thread!");
                }
            }
        }
        Message::ExternalMetadataResult(metadata) => {
            let tx = guild_id_to_tx.get_mut(&metadata.guild_id);
            match tx {
                Some(tx) => {
                    let gt = tx.send_now(IPCData::MetadataResult(metadata.clone()));
                    match gt {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Failed to send error report with error: {:?}", e);
                        }
                    }
                }
                None => {
                    error!("Failed to get appropriate sender when attempting to send error report")
                }
            }
        }
        _ => {}
    }
}

pub async fn init_processor(
    mut to_bg_rx: UnboundedReceiver<IPCData>,
    mut from_bg_tx: UnboundedSender<IPCData>,
    consumer: BaseConsumer,
    mut producer: FutureProducer,
    config: CharcoalConfig,
) {
    let mut guild_id_to_tx: HashMap<String, Arc<UnboundedSender<IPCData>>> = HashMap::new();
    loop {
        let mss = consumer.poll(Duration::from_millis(25));
        if let Some(p) = mss {
            match p {
                Ok(m) => {
                    let payload = m.payload();

                    match payload {
                        Some(payload) => {
                            let parsed_message: Result<Message, serde_json::Error> =
                                serde_json::from_slice(payload);

                            match parsed_message {
                                Ok(m) => {
                                    parse_message(m, &mut guild_id_to_tx, &mut from_bg_tx).await;
                                }
                                Err(e) => error!("{}", e),
                            }
                        }
                        None => {
                            error!("Received No Payload!");
                        }
                    }
                }
                Err(e) => error!("{}", e),
            }
        }
        // Receive messages from main function
        let rx_data = to_bg_rx.try_next();
        match rx_data {
            Ok(d) => {
                if d.is_some() {
                    if let IPCData::FromMain(m) = d.unwrap() {
                        guild_id_to_tx.insert(m.guild_id, m.response_tx);
                        send_message(&m.message, &config.kafka_topic, &mut producer).await;
                    }
                }
            }
            Err(e) => {
                if e.to_string() == "channel empty" {
                    debug!("Channel empty!");
                } else {
                    error!("Receive failed with: {}", e);
                }
            }
        }
    }
}
