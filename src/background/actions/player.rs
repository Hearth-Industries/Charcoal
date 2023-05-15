use hearth_interconnect::messages::{JobRequest, Message};
use hearth_interconnect::worker_communication::{DirectWorkerCommunication, DWCActionType};
use kafka::consumer::Consumer;
use kafka::producer::Producer;
use log::error;
use nanoid::nanoid;
use snafu::Whatever;
use crate::connector::{boilerplate_parse_result, send_message};

async fn play_from_http(producer: &mut Producer, guild_id: String,job_id: String, url: String) {
    send_message(&Message::DirectWorkerCommunication(DirectWorkerCommunication {
        job_id,
        action_type: DWCActionType::PlayDirectLink,
        play_audio_url: Some(url),
        guild_id: Some(guild_id),
        request_id: Some(nanoid!()),
        new_volume: None,
        seek_position: None,
        loop_times: None,
    }),"communication",producer);

}
async fn play_from_youtube(producer: &mut Producer, guild_id: String,job_id: String, url: String) {
    send_message(&Message::DirectWorkerCommunication(DirectWorkerCommunication {
        job_id,
        action_type: DWCActionType::PlayFromYoutube,
        play_audio_url: Some(url),
        guild_id: Some(guild_id),
        request_id: Some(nanoid!()),
        new_volume: None,
        seek_position: None,
        loop_times: None,
    }),"communication",producer);

}