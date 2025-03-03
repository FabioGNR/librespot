use crate::{
    deserialize_with::*,
    protocol::{
        context::Context,
        context_player_options::ContextPlayerOptionOverrides,
        player::{PlayOrigin, ProvidedTrack},
        transfer_state::TransferState,
    },
};
use serde::Deserialize;
use serde_json::Value;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Deserialize)]
pub struct Request {
    pub message_id: u32,
    // todo: did only send target_alias_id: null so far, maybe we just ignore it, will see
    // pub target_alias_id: Option<()>,
    pub sent_by_device_id: String,
    pub command: Command,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "endpoint", rename_all = "snake_case")]
pub enum Command {
    Transfer(TransferCommand),
    #[serde(deserialize_with = "boxed")]
    Play(Box<PlayCommand>),
    Pause(PauseCommand),
    SeekTo(SeekToCommand),
    SetShufflingContext(SetValueCommand),
    SetRepeatingTrack(SetValueCommand),
    SetRepeatingContext(SetValueCommand),
    AddToQueue(AddToQueueCommand),
    SetQueue(SetQueueCommand),
    SetOptions(SetOptionsCommand),
    UpdateContext(UpdateContextCommand),
    SkipNext(SkipNextCommand),
    // commands that don't send any context (at least not usually...)
    SkipPrev(GenericCommand),
    Resume(GenericCommand),
    // catch unknown commands, so that we can implement them later
    #[serde(untagged)]
    Unknown(Value),
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Command::*;

        write!(
            f,
            "endpoint: {}{}",
            matches!(self, Unknown(_))
                .then_some("unknown ")
                .unwrap_or_default(),
            match self {
                Transfer(_) => "transfer",
                Play(_) => "play",
                Pause(_) => "pause",
                SeekTo(_) => "seek_to",
                SetShufflingContext(_) => "set_shuffling_context",
                SetRepeatingContext(_) => "set_repeating_context",
                SetRepeatingTrack(_) => "set_repeating_track",
                AddToQueue(_) => "add_to_queue",
                SetQueue(_) => "set_queue",
                SetOptions(_) => "set_options",
                UpdateContext(_) => "update_context",
                SkipNext(_) => "skip_next",
                SkipPrev(_) => "skip_prev",
                Resume(_) => "resume",
                Unknown(json) => {
                    json.as_object()
                        .and_then(|obj| obj.get("endpoint").map(|v| v.as_str()))
                        .flatten()
                        .unwrap_or("???")
                }
            }
        )
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TransferCommand {
    #[serde(default, deserialize_with = "base64_proto")]
    pub data: Option<TransferState>,
    pub options: TransferOptions,
    pub from_device_identifier: String,
    pub logging_params: LoggingParams,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PlayCommand {
    #[serde(deserialize_with = "json_proto")]
    pub context: Context,
    #[serde(deserialize_with = "json_proto")]
    pub play_origin: PlayOrigin,
    pub options: PlayOptions,
    pub logging_params: LoggingParams,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PauseCommand {
    // does send options with it, but seems to be empty, investigate which options are send here
    pub logging_params: LoggingParams,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SeekToCommand {
    pub value: u32,
    pub position: u32,
    pub logging_params: LoggingParams,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SkipNextCommand {
    #[serde(default, deserialize_with = "option_json_proto")]
    pub track: Option<ProvidedTrack>,
    pub logging_params: LoggingParams,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SetValueCommand {
    pub value: bool,
    pub logging_params: LoggingParams,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AddToQueueCommand {
    #[serde(deserialize_with = "json_proto")]
    pub track: ProvidedTrack,
    pub logging_params: LoggingParams,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SetQueueCommand {
    #[serde(deserialize_with = "vec_json_proto")]
    pub next_tracks: Vec<ProvidedTrack>,
    #[serde(deserialize_with = "vec_json_proto")]
    pub prev_tracks: Vec<ProvidedTrack>,
    // this queue revision is actually the last revision, so using it will not update the web ui
    // might be that internally they use the last revision to create the next revision
    pub queue_revision: String,
    pub logging_params: LoggingParams,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SetOptionsCommand {
    pub shuffling_context: Option<bool>,
    pub repeating_context: Option<bool>,
    pub repeating_track: Option<bool>,
    pub options: Option<OptionsOptions>,
    pub logging_params: LoggingParams,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateContextCommand {
    #[serde(deserialize_with = "json_proto")]
    pub context: Context,
    pub session_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GenericCommand {
    pub logging_params: LoggingParams,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TransferOptions {
    pub restore_paused: String,
    pub restore_position: String,
    pub restore_track: String,
    pub retain_session: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PlayOptions {
    pub skip_to: Option<SkipTo>,
    #[serde(default, deserialize_with = "option_json_proto")]
    pub player_options_override: Option<ContextPlayerOptionOverrides>,
    pub license: Option<String>,
    // possible to send wie web-api
    pub seek_to: Option<u32>,
    // mobile
    pub always_play_something: Option<bool>,
    pub audio_stream: Option<String>,
    pub initially_paused: Option<bool>,
    pub prefetch_level: Option<String>,
    pub system_initiated: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OptionsOptions {
    only_for_local_device: bool,
    override_restrictions: bool,
    system_initiated: bool,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct SkipTo {
    pub track_uid: Option<String>,
    pub track_uri: Option<String>,
    pub track_index: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoggingParams {
    pub interaction_ids: Option<Vec<String>>,
    pub device_identifier: Option<String>,
    pub command_initiated_time: Option<i64>,
    pub page_instance_ids: Option<Vec<String>>,
    pub command_id: Option<String>,
}
