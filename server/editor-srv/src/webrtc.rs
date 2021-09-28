use std::{io::Write, sync::Arc, time::Duration};

use interceptor::registry::Registry;
use legion_mp4::Mp4Stream;

use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors, media_engine::MediaEngine, APIBuilder,
        API,
    },
    data::data_channel::{data_channel_message::DataChannelMessage, RTCDataChannel},
    peer::{
        configuration::RTCConfiguration, ice::ice_server::RTCIceServer,
        peer_connection::RTCPeerConnection, peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription,
    },
};

use legion_codec_api::{backends::openh264::encoder, formats};

pub struct WebRTCServer {
    api: API,
    peer_connections: Vec<RTCPeerConnection>,
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

        Ok(WebRTCServer {
            api,
            peer_connections: vec![],
        })
    }

    pub async fn new_peer_connection(&mut self) -> Result<&mut RTCPeerConnection, anyhow::Error> {
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
                println!("Peer connection state has changed: {}", s);

                Box::pin(async {})
            }))
            .await;

        // Register data channel creation handling
        peer_connection
            .on_data_channel(Box::new(
                move |data_channel: Arc<RTCDataChannel>| match data_channel.label() {
                    "control" => Box::pin(async move {
                        Self::handle_control_data_channel(data_channel)
                            .await
                            .unwrap()
                    }),
                    "video" => Box::pin(async move {
                        Self::handle_video_data_channel(data_channel).await.unwrap()
                    }),
                    _ => Box::pin(
                        async move { Self::ignore_data_channel(data_channel).await.unwrap() },
                    ),
                },
            ))
            .await;

        self.peer_connections.push(peer_connection);

        Ok(self.peer_connections.last_mut().unwrap())
    }

    async fn ignore_data_channel(data_channel: Arc<RTCDataChannel>) -> anyhow::Result<()> {
        println!(
            "Ignoring unknown data channel type `{}`.",
            data_channel.label()
        );

        data_channel.close().await
    }

    async fn handle_control_data_channel(_data_channel: Arc<RTCDataChannel>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn handle_video_data_channel(data_channel: Arc<RTCDataChannel>) -> anyhow::Result<()> {
        // Sample code to stream a video.
        let src = &include_bytes!("../assets/lenna_512x512.rgb")[..];

        let config = encoder::EncoderConfig::new(512, 512).debug(true);
        let mut encoder = encoder::Encoder::with_config(config)?;
        let mut converter = formats::RBGYUVConverter::new(512, 512);
        let mut converter2 = formats::RBGYUVConverter::new(512, 512);

        converter.convert(src);
        converter2.convert2(src);

        let on_close_name = data_channel.name();

        data_channel
            .on_close(Box::new(move || {
                println!("Video data channel {} closed.", on_close_name);

                Box::pin(async {})
            }))
            .await;

        let on_open_data_channel = Arc::clone(&data_channel);

        data_channel
            .on_open(Box::new(move || {
                println!("Video data channel opened.");

                let data_channel = on_open_data_channel;
                Box::pin(async move {
                    let mut file = std::fs::File::create("/mnt/d/test.mp4").unwrap();
                    let mut mp4 = Mp4Stream::new(30);
                    let track_id = mp4.add_track(512, 512).unwrap();
                    mp4.set_sps(
                        track_id,
                        &[
                            103, 66, 192, 31, 140, 141, 64, 64, 8, 52, 3, 194, 33, 26, 128,
                        ],
                    )
                    .unwrap();
                    mp4.set_pps(track_id, &[104, 206, 60, 128]).unwrap();
                    if data_channel
                        .send(&bytes::Bytes::copy_from_slice(mp4.get_content()))
                        .await
                        .is_err()
                    {
                        println!("Failed to send sample {}: streaming will stop.", 0)
                    }
                    file.write_all(mp4.get_content()).unwrap();
                    mp4.clean();
                    for sample_id in 0..100 {
                        let stream = if sample_id % 2 == 0 {
                            encoder.encode(&converter, sample_id % 8 == 0).unwrap()
                        } else {
                            encoder.encode(&converter2, sample_id % 8 == 0).unwrap()
                        };

                        for layer in &stream.layers {
                            if !layer.is_video {
                                continue;
                            }
                            for nalu in &layer.nal_units {
                                let size = nalu.len() - 4;
                                let mut vec = vec![];
                                vec.extend_from_slice(nalu);
                                vec[0] = (size >> 24) as u8;
                                vec[1] = ((size >> 16) & 0xFF) as u8;
                                vec[2] = ((size >> 8) & 0xFF) as u8;
                                vec[3] = (size & 0xFF) as u8;

                                mp4.add_frame(track_id, sample_id % 8 == 0, &vec).unwrap();
                            }
                        }
                        if data_channel
                            .send(&bytes::Bytes::copy_from_slice(mp4.get_content()))
                            .await
                            .is_err()
                        {
                            println!("Failed to send sample {}: streaming will stop.", sample_id)
                        }
                        file.write_all(mp4.get_content()).unwrap();
                        mp4.clean();
                        // Wait the right time before the next frame.
                        let timeout = tokio::time::sleep(Duration::from_millis(
                            ((16 * 1000) as f64 / 60.0).round() as u64,
                        ));
                        tokio::pin!(timeout);

                        tokio::select! {
                            _ = timeout.as_mut() =>{
                            }
                        };

                        println!("Sent sample {}.", sample_id)
                    }
                })
            }))
            .await;

        let on_message_name = data_channel.name();

        // Register text message handling
        data_channel
            .on_message(Box::new(move |msg: DataChannelMessage| {
                let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                println!("{}: {}", on_message_name, msg_str);

                Box::pin(async {})
            }))
            .await;

        Ok(())
    }

    pub async fn initialize_stream(
        &mut self,
        remote_rtc_session_description: Vec<u8>,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let offer =
            serde_json::from_slice::<RTCSessionDescription>(&remote_rtc_session_description)?;

        // Clear out old connections as we only allow one active connection to the current instance right now.
        for old_peer_connection in self.peer_connections.iter() {
            let _ = old_peer_connection.close().await;
        }
        self.peer_connections.clear();

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

        Ok(serde_json::to_vec(&rtc_session_description)?)
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
