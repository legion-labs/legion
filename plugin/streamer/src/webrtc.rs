use std::sync::Arc;

use interceptor::registry::Registry;
use log::{debug, info, warn};
use tokio::sync::Notify;
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors,
        media_engine::{MediaEngine, MIME_TYPE_H264},
        APIBuilder, API,
    },
    data::data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel},
    media::{
        rtp::rtp_codec::{RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType},
        track::track_local::{track_local_static_sample::TrackLocalStaticSample, TrackLocal},
    },
    peer::{
        configuration::RTCConfiguration,
        ice::{ice_candidate::RTCIceCandidate, ice_server::RTCIceServer},
        peer_connection::RTCPeerConnection,
        peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription,
        signaling_state::RTCSignalingState,
    },
};

use super::streamer::{StreamEvent, StreamID};

/// `WebRTCServer` implements a fully-compliant `WebRTC` server that can
/// establish peer-to-peer connections with several hosts for streaming
/// purposes.
pub(crate) struct WebRTCServer {
    api: API,
}

impl WebRTCServer {
    /// Instanciate a new `WebRTCServer` with its own media engin and
    /// interceptors registry.
    ///
    /// Typically, a single `WebRTCServer` is enough for any given application.
    pub(crate) fn new() -> Result<Self, anyhow::Error> {
        // Create a MediaEngine object to configure the supported codec
        let mut media_engine = MediaEngine::default();

        // Register default codecs
        media_engine.register_default_codecs()?;

        media_engine.register_codec(
            RTCRtpCodecParameters {
                capability: RTCRtpCodecCapability {
                    mime_type: "video/mp4; codecs=\"avc1.640C34\"".to_owned(),
                    ..RTCRtpCodecCapability::default()
                },
                ..RTCRtpCodecParameters::default()
            },
            RTPCodecType::Video,
        )?;

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

    pub(crate) async fn initialize_stream_connection(
        &self,
        remote_rtc_session_description: Vec<u8>,
        stream_events_sender: Arc<crossbeam::channel::Sender<StreamEvent>>,
        stream_ice_candidates_sender: Arc<crossbeam::channel::Sender<RTCIceCandidate>>,
    ) -> Result<(StreamID, Arc<RTCPeerConnection>, Vec<u8>), anyhow::Error> {
        let offer =
            serde_json::from_slice::<RTCSessionDescription>(&remote_rtc_session_description)?;

        let (stream_id, peer_connection) = self
            .new_peer_connection(stream_events_sender, stream_ice_candidates_sender)
            .await?;

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

        Ok((
            stream_id,
            peer_connection,
            serde_json::to_vec(&rtc_session_description)?,
        ))
    }

    async fn new_peer_connection(
        &self,
        stream_events_sender: Arc<crossbeam::channel::Sender<StreamEvent>>,
        stream_ice_candidates_sender: Arc<crossbeam::channel::Sender<RTCIceCandidate>>,
    ) -> anyhow::Result<(StreamID, Arc<RTCPeerConnection>)> {
        // Prepare the configuration
        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..RTCIceServer::default()
            }],
            ..RTCConfiguration::default()
        };

        // Will notify video streamer when the peer connection is ready
        let notify_video = Arc::new(Notify::new());

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
        let on_state_change_notify_video = Arc::clone(&notify_video);

        peer_connection
            .on_peer_connection_state_change(Box::new(
                move |connection_state: RTCPeerConnectionState| {
                    info!("Peer connection state has changed: {}", connection_state);

                    match connection_state {
                        RTCPeerConnectionState::Connected => {
                            on_state_change_notify_video.notify_waiters();
                        }
                        RTCPeerConnectionState::Disconnected => {
                            let _ = on_state_change_stream_events_sender.send(
                                StreamEvent::ConnectionClosed(
                                    stream_id,
                                    Arc::clone(&on_state_change_peer_connection),
                                ),
                            );
                        }
                        _ => {}
                    }

                    Box::pin(async {})
                },
            ))
            .await;

        let on_data_channel_stream_events_sender = Arc::clone(&stream_events_sender);

        // Register data channel creation handling
        peer_connection
            .on_data_channel(Box::new(move |data_channel: Arc<RTCDataChannel>| {
                let on_data_channel_stream_events_sender =
                    Arc::clone(&on_data_channel_stream_events_sender);

                match data_channel.label() {
                    "control" => Box::pin(async move {
                        Self::handle_control_data_channel(
                            data_channel,
                            stream_id,
                            on_data_channel_stream_events_sender,
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

        let on_ice_candidate_stream_ice_candidates_sender =
            Arc::clone(&stream_ice_candidates_sender);

        // Send ice candidates to a client
        peer_connection
            .on_ice_candidate(Box::new(move |ice_candidate| {
                let on_ice_candidate_stream_ice_candidates_sender =
                    Arc::clone(&on_ice_candidate_stream_ice_candidates_sender);

                Box::pin(async move {
                    if let Some(ice_candidate) = ice_candidate {
                        on_ice_candidate_stream_ice_candidates_sender
                            .send(ice_candidate)
                            .unwrap();
                    }
                })
            }))
            .await;

        let video_track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_string(),
                // mime_type: "video/mp4; codecs=\"avc1.640C34\"".to_string(),
                ..RTCRtpCodecCapability::default()
            },
            "video".to_string(),
            stream_id.to_string(),
        ));

        let rtp_sender = peer_connection
            .add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>)
            .await?;

        tokio::spawn(async move {
            let mut rtcp_buf = vec![0u8; 1500];

            while let Ok((i, hmap)) = rtp_sender.read(&mut rtcp_buf).await {
                debug!("Rtp index: {} - map: {:?}", i, hmap);
            }
        });

        let notify_video = Arc::clone(&notify_video);

        tokio::spawn(async move {
            notify_video.notified().await;

            // stream_events_sender
            //     .send(StreamEvent::VideoChannelOpened(stream_id, video_track))
            //     .unwrap();
            crate::streamer::video_stream::VideoStream::write_video_to_track(
                "../../plugin/streamer/sample.h264",
                video_track,
            )
            .await
            .unwrap();
        });

        Ok((stream_id, peer_connection))
    }

    async fn ignore_data_channel(data_channel: Arc<RTCDataChannel>) -> Result<(), webrtc::Error> {
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
}

trait RTCDataChannelID {
    fn name(&self) -> String;
}

impl RTCDataChannelID for webrtc::data::data_channel::RTCDataChannel {
    fn name(&self) -> String {
        format!("{}-{}", self.label(), self.id())
    }
}
