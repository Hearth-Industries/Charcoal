use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use hearth_interconnect::worker_communication::{DirectWorkerCommunication, DWCActionType};
use log::error;
use crate::{InternalIPC, InternalIPCType, PlayerObject};
use crate::actions::helpers::send_direct_worker_communication;

#[async_trait]
pub trait TrackManager {
    async fn set_playback_volume(&self,playback_volume: f32);
    async fn force_stop_loop(&self);
    async fn loop_indefinitely(&self);
    async fn loop_x_times(&self,times: usize);
    async fn seek_to_position(&self,position: Duration);
    async fn resume_playback(&self);
    async fn pause_playback(&self);
}
#[async_trait]
impl TrackManager for PlayerObject {
    async fn set_playback_volume(&self,playback_volume: f32) {
        let action_type = DWCActionType::SetPlaybackVolume;
        //
        let mut charcoal = self.charcoal.lock().await;
        send_direct_worker_communication(&mut charcoal.producer,DirectWorkerCommunication {
            job_id: self.job_id.clone().unwrap(),
            action_type: action_type.clone(),
            play_audio_url: None,
            guild_id: Some(self.guild_id.clone().unwrap()),
            request_id: None,
            new_volume: Some(playback_volume),
            seek_position: None,
            loop_times: None,
        },self).await;
    }
    async fn force_stop_loop(&self) {
        let action_type = DWCActionType::ForceStopLoop;
        //
        let mut charcoal = self.charcoal.lock().await;
        send_direct_worker_communication(&mut charcoal.producer,DirectWorkerCommunication {
            job_id: self.job_id.clone().unwrap(),
            action_type: action_type.clone(),
            play_audio_url: None,
            guild_id: Some(self.guild_id.clone().unwrap()),
            request_id: None,
            new_volume: None,
            seek_position: None,
            loop_times: None,
        },self).await;
    }
    async fn loop_indefinitely(&self) {
        let action_type = DWCActionType::LoopForever;
        //
        let mut charcoal = self.charcoal.lock().await;
        send_direct_worker_communication(&mut charcoal.producer,DirectWorkerCommunication {
            job_id: self.job_id.clone().unwrap(),
            action_type: action_type.clone(),
            play_audio_url: None,
            guild_id: Some(self.guild_id.clone().unwrap()),
            request_id: None,
            new_volume: None,
            seek_position: None,
            loop_times: None,
        },self).await;
    }
    async fn loop_x_times(&self,times: usize) {
        let action_type = DWCActionType::LoopXTimes;
        //
        let mut charcoal = self.charcoal.lock().await;
        send_direct_worker_communication(&mut charcoal.producer,DirectWorkerCommunication {
            job_id: self.job_id.clone().unwrap(),
            action_type: action_type.clone(),
            play_audio_url: None,
            guild_id: Some(self.guild_id.clone().unwrap()),
            request_id: None,
            new_volume: None,
            seek_position: None,
            loop_times: Some(times.clone()),
        },self).await;
    }
    async fn seek_to_position(&self,position: Duration) {
        let action_type = DWCActionType::SeekToPosition;
        //
        let mut charcoal = self.charcoal.lock().await;
        send_direct_worker_communication(&mut charcoal.producer,DirectWorkerCommunication {
            job_id: self.job_id.clone().unwrap(),
            action_type: action_type.clone(),
            play_audio_url: None,
            guild_id: Some(self.guild_id.clone().unwrap()),
            request_id: None,
            new_volume: None,
            seek_position: Some(position.as_millis() as u64),
            loop_times: None,
        },self).await;
    }
    async fn resume_playback(&self) {
        let action_type = DWCActionType::ResumePlayback;
        //
        let mut charcoal = self.charcoal.lock().await;
        send_direct_worker_communication(&mut charcoal.producer,DirectWorkerCommunication {
            job_id: self.job_id.clone().unwrap(),
            action_type: action_type.clone(),
            play_audio_url: None,
            guild_id: Some(self.guild_id.clone().unwrap()),
            request_id: None,
            new_volume: None,
            seek_position: None,
            loop_times: None,
        },self).await;
    }
    async fn pause_playback(&self) {
        let action_type = DWCActionType::PausePlayback;
        //
        let mut charcoal = self.charcoal.lock().await;
        send_direct_worker_communication(&mut charcoal.producer,DirectWorkerCommunication {
            job_id: self.job_id.clone().unwrap(),
            action_type: action_type.clone(),
            play_audio_url: None,
            guild_id: Some(self.guild_id.clone().unwrap()),
            request_id: None,
            new_volume: None,
            seek_position: None,
            loop_times: None,
        },self).await;
    }
}