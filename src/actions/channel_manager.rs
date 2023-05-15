use std::sync::Arc;
use hearth_interconnect::messages::{JobRequest, Message};
use hearth_interconnect::worker_communication::{DirectWorkerCommunication, DWCActionType};
use log::{debug, error};
use nanoid::nanoid;
use crate::{InfrastructureType, InternalIPC, InternalIPCType, PlayerObject, StandardActionType};
use async_trait::async_trait;
use crate::connector::{ boilerplate_parse_result, send_message};

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
        // Parse result
        boilerplate_parse_result(|message| {
            match message {
                Message::ErrorReport(error_report) => {
                    error!("{} - Error with Job ID: {} and Request ID: {}",error_report.error,error_report.job_id,error_report.request_id);
                    return false;
                },
                Message::ExternalQueueJobResponse(res) => {
                    self.worker_id = Some(res.worker_id);
                    self.job_id = Some(res.job_id);
                    return false;
                },
                _ => {}
            }
            return true;
        },&mut self.charcoal.lock().await.consumer)
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