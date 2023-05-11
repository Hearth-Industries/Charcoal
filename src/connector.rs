// Internal connector



use std::process;

use std::time::Duration;
use hearth_interconnect::messages::{Message, MessageType};
use kafka;
use kafka::consumer::Consumer;
use kafka::producer::{Producer, Record, RequiredAcks};
use log::{debug, error, info, warn};
use nanoid::nanoid;
use openssl;
use snafu::Whatever;
use tokio::sync::broadcast::{Receiver, Sender};
use crate::{InternalIPC, InternalIPCType, StandardActionType};
use self::kafka::client::{FetchOffset, KafkaClient, SecurityConfig};
use self::openssl::ssl::{SslConnector, SslFiletype, SslMethod, SslVerifyMode};


pub fn init_connector(broker: String,sender: Sender<InternalIPC>,receiver: Receiver<InternalIPC>) {

    let brokers = vec![broker];

    let producer : Producer = initialize_producer(initialize_client(&brokers));

    initialize_consume(brokers,producer,sender,receiver);
}

pub fn initialize_client(brokers: &Vec<String>) -> KafkaClient {
    // ~ OpenSSL offers a variety of complex configurations. Here is an example:
    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    builder.set_cipher_list("DEFAULT").unwrap();
    builder.set_verify(SslVerifyMode::PEER);

    let cert_file = "service.cert";
    let cert_key = "service.key";
    let ca_cert = "ca.pem";

    info!("loading cert-file={}, key-file={}", cert_file, cert_key);

    builder
        .set_certificate_file(cert_file, SslFiletype::PEM)
        .unwrap();
    builder
        .set_private_key_file(cert_key, SslFiletype::PEM)
        .unwrap();
    builder.check_private_key().unwrap();

    builder.set_ca_file(ca_cert).unwrap();

    let connector = builder.build();

    // ~ instantiate KafkaClient with the previous OpenSSL setup
    let mut client = KafkaClient::new_secure(
        brokers.to_owned(),
        SecurityConfig::new(connector)
    );

    // ~ communicate with the brokers
    match client.load_metadata_all() {
        Err(e) => {
            error!("{:?}", e);
            drop(client);
            process::exit(1);
        }
        Ok(_) => {
            // ~ at this point we have successfully loaded
            // metadata via a secured connection to one of the
            // specified brokers

            if client.topics().len() == 0 {
                warn!("No topics available!");
            } else {
                // ~ now let's communicate with all the brokers in
                // the cluster our topics are spread over

                let topics: Vec<String> = client.topics().names().map(Into::into).collect();
                match client.fetch_offsets(topics.as_slice(), FetchOffset::Latest) {
                    Err(e) => {
                        error!("{:?}", e);
                        drop(client);
                        process::exit(1);
                    }
                    Ok(toffsets) => {
                        debug!("Topic offsets:");
                        for (topic, mut offs) in toffsets {
                            offs.sort_by_key(|x| x.partition);
                            debug!("{}", topic);
                            for off in offs {
                                debug!("\t{}: {:?}", off.partition, off.offset);
                            }
                        }
                    }
                }
            }
        }
    }
    return client;
}

pub fn initialize_producer(client: KafkaClient) -> Producer {
    let producer = Producer::from_client(client)
        // ~ give the brokers one second time to ack the message
        .with_ack_timeout(Duration::from_secs(1))
        // ~ require only one broker to ack the message
        .with_required_acks(RequiredAcks::One)
        // ~ build the producer with the above settings
        .create().unwrap();
    return producer;
}

fn parse_message(parsed_message: Message, _producer: &mut Producer) -> Result<(),Whatever> {
    match parsed_message.message_type {
        MessageType::ErrorReport => {
            let error_report = parsed_message.error_report.unwrap();
            error!("{} - Error with Job ID: {} and Request ID: {}",error_report.error,error_report.job_id,error_report.request_id)
        }
        _ => {}
    }
    Ok(())
}

pub fn initialize_consume(brokers: Vec<String>, mut producer: Producer, _tx: Sender<InternalIPC>, mut rx: Receiver<InternalIPC>) {
    let mut consumer = Consumer::from_client(initialize_client(&brokers))
        .with_topic(String::from("communication"))
        .create()
        .unwrap();

    loop {
        let mss = consumer.poll().unwrap();
        if mss.is_empty() {
            debug!("No messages available right now.");
        }

        for ms in mss.iter() {
            for m in ms.messages() {
                let parsed_message : Result<Message,serde_json::Error> = serde_json::from_slice(&m.value);
                match parsed_message {
                    Ok(message) => {
                        let parse = parse_message(message,&mut producer);
                        match parse {
                            Ok(_) => {},
                            Err(e) => error!("Failed to parse message with error: {}",e)
                        }
                    },
                    Err(e) => error!("{} - Failed to parse message",e),
                }
            }
            let _ = consumer.consume_messageset(ms);
        }
        consumer.commit_consumed().unwrap();

        let res = rx.try_recv();
        match res {
            Ok(msg) => {
                let request_id = nanoid!();
                match msg.action {
                    InternalIPCType::DWCAction(..) => {
                        send_message(&Message {
                            message_type: MessageType::DirectWorkerCommunication,
                            analytics: None,
                            queue_job_request: None,
                            queue_job_internal: None,
                            request_id: request_id.clone(),
                            worker_id: None,
                            direct_worker_communication: msg.dwc,
                            external_queue_job_response: None,
                            job_event: None,
                            error_report: None,
                        },"communication",&mut producer)
                    },
                    InternalIPCType::StandardAction(StandardActionType::JoinChannel) => {
                        send_message(&Message {
                            message_type: MessageType::DirectWorkerCommunication,
                            analytics: None,
                            queue_job_request: msg.queue_job_request,
                            queue_job_internal: None,
                            request_id: request_id.clone(),
                            worker_id: None,
                            direct_worker_communication: None,
                            external_queue_job_response: None,
                            job_event: None,
                            error_report: None,
                        },"communication",&mut producer)
                    }
                }
            },
            Err(_e) => {}
        }
    }
}

pub fn send_message(message: &Message, topic: &str, producer: &mut Producer) {
    // Send message to worker
    let data = serde_json::to_string(message).unwrap();
    producer.send(&Record::from_value(topic, data)).unwrap();
}
