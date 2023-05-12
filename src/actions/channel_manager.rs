use std::sync::Arc;
use hearth_interconnect::messages::{JobRequest, Message};
use hearth_interconnect::worker_communication::{DirectWorkerCommunication, DWCActionType};
use log::{debug, error};
use nanoid::nanoid;
use crate::{InfrastructureType, InternalIPC, InternalIPCType, PlayerObject, StandardActionType};
use async_trait::async_trait;
use crate::connector::send_message;

#[async_trait]
pub trait ChannelManager {
    async fn join_channel(&mut self,guild_id: String,voice_channel_id: String);
    async fn exit_channel(&self);
}

#[async_trait]
impl ChannelManager for PlayerObject {
    async fn join_channel(&mut self, guild_id: String, voice_channel_id: String) {
        self.guild_id = Some(guild_id.clone());
        let mut charcoal = self.charcoal.lock().await;
        send_message(&Message::ExternalQueueJob(JobRequest {
            guild_id,
            voice_channel_id,
            request_id: nanoid!(),
        }), "communication", &mut charcoal.producer);
        //
        let mut check_result = true;
        while check_result {
            let mss = charcoal.consumer.poll().unwrap();
            if mss.is_empty() {
                debug!("No messages available right now.");
            }

            for ms in mss.iter() {
                for m in ms.messages() {
                    let parsed_message : Result<Message,serde_json::Error> = serde_json::from_slice(&m.value);
                    match parsed_message {
                        Ok(message) => {
                            match message {
                                Message::ErrorReport(error_report) => {
                                    error!("{} - Error with Job ID: {} and Request ID: {}",error_report.error,error_report.job_id,error_report.request_id)
                                },
                                Message::ExternalQueueJobResponse(res) => {
                                    self.worker_id = Some(res.worker_id);
                                    self.job_id = Some(res.job_id);
                                    check_result = false;
                                },
                                _ => {}
                            }
                        },
                        Err(e) => error!("{} - Failed to parse message",e),
                    }
                }
                let _ = charcoal.consumer.consume_messageset(ms);
            }
            charcoal.consumer.commit_consumed().unwrap();
        }
    }
    async fn exit_channel(&self) {
        let mut charcoal = self.charcoal.lock().await;
        send_message(&Message::DirectWorkerCommunication(DirectWorkerCommunication {
            job_id: self.job_id.clone().unwrap(),
            action_type: DWCActionType::LeaveChannel,
            play_audio_url: None,
            guild_id: Some(self.guild_id.clone().unwrap()),
            request_id: None,
            new_volume: None,
            seek_position: None,
            loop_times: None,
        }),"communication",&mut charcoal.producer);
    }
}