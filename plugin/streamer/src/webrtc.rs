use log::{debug, info, warn};
use std::sync::Arc;

use interceptor::registry::Registry;

use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
        API,
    },
    data::data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel},
    peer::{
        configuration::RTCConfiguration, ice::ice_server::RTCIceServer,
        peer_connection::RTCPeerConnection, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription, signaling_state::RTCSignalingState,
    },
};

use super::streamer::{StreamEvent, StreamID};

/// `WebRTCServer` implements a fully-compliant `WebRTC` server that can
/// establish peer-to-peer connections with several hosts for streaming
/// purposes.
pub struct WebRTCServer {
    api: API,
}

impl WebRTCServer {
    /// Instanciate a new `WebRTCServer` with its own media engin and
    /// interceptors registry.
    ///
    /// Typically, a single `WebRTCServer` is enough for any given application.
    pub fn new() -> Result<Self, anyhow::Error> {
        // Create a MediaEngine object to configure the supported codec
        let mut media_engine = MediaEngine::default();

        // Register default codecs
        media_engine.register_default_codecs()?;

        // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
        // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
        // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
        // for each PeerConnection.
        let mut registry = Registry::new();

        // Use the default set of Interceptors
        registry = register_default_interceptors(registry, &mut media_engine)?;

        // Create the API object with the MediaEngine
        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .build();

        Ok(Self { api })
    }

    pub async fn initialize_stream_connection(
        &self,
        remote_rtc_session_description: Vec<u8>,
        stream_events_sender: Arc<crossbeam::channel::Sender<StreamEvent>>,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let offer =
            serde_json::from_slice::<RTCSessionDescription>(&remote_rtc_session_description)?;

        let peer_connection = self.new_peer_connection(stream_events_sender).await?;

        // Set the remote SessionDescription
        peer_connection.set_remote_description(offer).await?;

        // Create an answer
        let answer = peer_connection.create_answer(None).await?;

        // Create channel that is blocked until ICE Gathering is complete
        let mut gather_complete = peer_connection.gathering_complete_promise().await;

        // Sets the LocalDescription, and starts our UDP listeners
        peer_connection.set_local_description(answer).await?;

        // Block until ICE Gathering is complete, disabling trickle ICE
        // we do this because we only can exchange one signaling message
        // in a production application you should exchange ICE Candidates via OnICECandidate
        let _ = gather_complete.recv().await;

        // Output the answer in base64 so we can paste it in browser
        let rtc_session_description = peer_connection.local_description().await.unwrap();

        Ok(serde_json::to_vec(&rtc_session_description)?)
    }

    async fn new_peer_connection(
        &self,
        stream_events_sender: Arc<crossbeam::channel::Sender<StreamEvent>>,
    ) -> anyhow::Result<Arc<RTCPeerConnection>> {
        // Prepare the configuration
        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let peer_connection = Arc::new(self.api.new_peer_connection(config).await?);

        peer_connection
            .on_signaling_state_change(Box::new(|s: RTCSignalingState| {
                info!("Peer connection signaling state has changed: {}", s);

                Box::pin(async {})
            }))
            .await;

        let (sender, receiver) = tokio::sync::oneshot::channel();

        let _ = stream_events_sender.send(StreamEvent::ConnectionEstablished(
            Arc::clone(&peer_connection),
            sender,
        ));

        let stream_id = receiver.await?;

        let on_state_change_peer_connection = Arc::clone(&peer_connection);
        let on_state_change_stream_events_sender = Arc::clone(&stream_events_sender);

        peer_connection
            .on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
                info!("Peer connection state has changed: {}", s);

                if s == RTCPeerConnectionState::Disconnected {
                    let _ =
                        on_state_change_stream_events_sender.send(StreamEvent::ConnectionClosed(
                            stream_id,
                            Arc::clone(&on_state_change_peer_connection),
                        ));
                };

                Box::pin(async {})
            }))
            .await;

        // Register data channel creation handling
        peer_connection
            .on_data_channel(Box::new(move |data_channel: Arc<RTCDataChannel>| {
                let stream_events_sender = Arc::clone(&stream_events_sender);

                match data_channel.label() {
                    "control" => Box::pin(async move {
                        Self::handle_control_data_channel(
                            data_channel,
                            stream_id,
                            stream_events_sender,
                        )
                        .await
                        .unwrap();
                    }),
                    "video" => Box::pin(async move {
                        Self::handle_video_data_channel(
                            data_channel,
                            stream_id,
                            stream_events_sender,
                        )
                        .await
                        .unwrap();
                    }),
                    _ => Box::pin(
                        async move { Self::ignore_data_channel(data_channel).await.unwrap() },
                    ),
                }
            }))
            .await;

        Ok(peer_connection)
    }

    async fn ignore_data_channel(data_channel: Arc<RTCDataChannel>) -> anyhow::Result<()> {
        warn!(
            "Ignoring unknown data channel type `{}`.",
            data_channel.label()
        );

        data_channel.close().await
    }

    async fn handle_control_data_channel(
        data_channel: Arc<RTCDataChannel>,
        stream_id: StreamID,
        stream_events_sender: Arc<crossbeam::channel::Sender<StreamEvent>>,
    ) -> anyhow::Result<()> {
        let on_open_stream_events_sender = Arc::clone(&stream_events_sender);
        let on_open_data_channel = Arc::clone(&data_channel);

        data_channel
            .on_open(Box::new(move || {
                info!("Control data channel opened.");

                let _ = on_open_stream_events_sender.send(StreamEvent::ControlChannelOpened(
                    stream_id,
                    on_open_data_channel,
                ));
                Box::pin(async {})
            }))
            .await;

        let on_close_stream_events_sender = Arc::clone(&stream_events_sender);
        let on_close_data_channel = Arc::clone(&data_channel);

        data_channel
            .on_close(Box::new(move || {
                info!("Control data channel closed.");

                let _ = on_close_stream_events_sender.send(StreamEvent::ControlChannelClosed(
                    stream_id,
                    Arc::clone(&on_close_data_channel),
                ));

                Box::pin(async {})
            }))
            .await;

        let on_message_name = data_channel.name();
        let on_message_stream_events_sender = Arc::clone(&stream_events_sender);
        let on_message_data_channel = Arc::clone(&data_channel);

        // Register text message handling
        data_channel
            .on_message(Box::new(move |msg: DataChannelMessage| {
                let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                debug!("{}: {}", on_message_name, msg_str);

                let _ = on_message_stream_events_sender.send(
                    StreamEvent::ControlChannelMessageReceived(
                        stream_id,
                        Arc::clone(&on_message_data_channel),
                        msg,
                    ),
                );

                Box::pin(async {})
            }))
            .await;

        Ok(())
    }

    async fn handle_video_data_channel(
        data_channel: Arc<RTCDataChannel>,
        stream_id: StreamID,
        stream_events_sender: Arc<crossbeam::channel::Sender<StreamEvent>>,
    ) -> anyhow::Result<()> {
        let on_error_name = data_channel.name();

        data_channel
            .on_error(Box::new(move |err| {
                warn!("Video data channel {} error: {}.", on_error_name, err);

                Box::pin(async {})
            }))
            .await;

        let on_open_stream_events_sender = Arc::clone(&stream_events_sender);
        let on_open_data_channel = Arc::clone(&data_channel);

        data_channel
            .on_open(Box::new(move || {
                info!("Video data channel opened.");

                let _ = on_open_stream_events_sender.send(StreamEvent::VideoChannelOpened(
                    stream_id,
                    on_open_data_channel,
                ));

                Box::pin(async move {})
            }))
            .await;

        let on_close_stream_events_sender = Arc::clone(&stream_events_sender);
        let on_close_data_channel = Arc::clone(&data_channel);

        data_channel
            .on_close(Box::new(move || {
                info!("Video data channel closed.");

                let _ = on_close_stream_events_sender.send(StreamEvent::VideoChannelClosed(
                    stream_id,
                    Arc::clone(&on_close_data_channel),
                ));

                Box::pin(async {})
            }))
            .await;

        let on_message_name = data_channel.name();
        let on_message_stream_events_sender = Arc::clone(&stream_events_sender);
        let on_message_data_channel = Arc::clone(&data_channel);

        // Register text message handling
        data_channel
            .on_message(Box::new(move |msg: DataChannelMessage| {
                let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                debug!("{}: {}", on_message_name, msg_str);

                let _ =
                    on_message_stream_events_sender.send(StreamEvent::VideoChannelMessageReceived(
                        stream_id,
                        Arc::clone(&on_message_data_channel),
                        msg,
                    ));

                Box::pin(async {})
            }))
            .await;

        Ok(())
    }
}

trait RTCDataChannelID {
    fn name(&self) -> String;
}

impl RTCDataChannelID for webrtc::data::data_channel::RTCDataChannel {
    fn name(&self) -> String {
        format!("{}-{}", self.label(), self.id())
    }
}
