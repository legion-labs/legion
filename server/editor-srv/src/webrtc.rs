use std::{sync::Arc, time::Duration};

use interceptor::registry::Registry;

use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
        API,
    },
    data::data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel},
    peer::{
        configuration::RTCConfiguration,
        ice::ice_server::RTCIceServer,
        peer_connection::RTCPeerConnection,
        peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::{RTCSessionDescription, RTCSessionDescriptionSerde},
    },
    util::math_rand_alpha,
};

pub struct WebRTCServer {
    api: API,
}

impl WebRTCServer {
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

        Ok(WebRTCServer { api })
    }

    pub async fn new_peer_connection(&self) -> Result<RTCPeerConnection, anyhow::Error> {
        // Prepare the configuration
        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };

        let peer_connection = self.api.new_peer_connection(config).await?;

        // Setup the various event handlers.
        //
        // The method for setting-up most event handlers are async because they
        // internally lock an async mutex.

        // Set the handler for Peer connection state
        // This will notify you when the peer has connected/disconnected
        peer_connection
            .on_peer_connection_state_change(Box::new(|s: RTCPeerConnectionState| {
                println!("Peer Connection State has changed: {}", s);

                if s == RTCPeerConnectionState::Failed {
                    // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                    // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                    // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                    println!("Peer Connection has gone to failed exiting");
                    std::process::exit(0);
                }

                Box::pin(async {})
            }))
            .await;

        // Register data channel creation handling
        peer_connection
        .on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
            let d_label = d.label().to_owned();
            let d_id = d.id();
            println!("New DataChannel {} {}", d_label, d_id);

            // Register channel opening handling
            Box::pin(async move {
                let d2 = Arc::clone(&d);
                let d_label2 = d_label.clone();
                let d_id2 = d_id;
                d.on_open(Box::new(move || {
                    println!("Data channel '{}'-'{}' open. Random messages will now be sent to any connected DataChannels every 5 seconds", d_label2, d_id2);

                    Box::pin(async move {
                        let mut result = Ok(0);
                        while result.is_ok() {
                            let timeout = tokio::time::sleep(Duration::from_secs(5));
                            tokio::pin!(timeout);

                            tokio::select! {
                                _ = timeout.as_mut() =>{
                                    let message = math_rand_alpha(15);
                                    println!("Sending '{}'", message);
                                    result = d2.send_text(message).await;
                                }
                            };
                        }
                    })
                })).await;

                // Register text message handling
                d.on_message(Box::new(move |msg: DataChannelMessage| {
                    let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                    println!("Message from DataChannel '{}': '{}'", d_label, msg_str);
                    Box::pin(async {})
                })).await;
            })
        }))
        .await;

        Ok(peer_connection)
    }

    pub async fn initialize_stream(
        &self,
        remote_rtc_session_description: Vec<u8>,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let mut offer = RTCSessionDescription::default();
        offer.serde =
            serde_json::from_slice::<RTCSessionDescriptionSerde>(&remote_rtc_session_description)?;

        let peer_connection = self.new_peer_connection().await?;

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

        Ok(serde_json::to_vec(&rtc_session_description.serde)?)
    }
}
