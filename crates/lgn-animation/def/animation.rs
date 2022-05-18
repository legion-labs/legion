use lgn_math::prelude::Vec3;

#[component]
struct AnimationComponent {
    #[legion(default = Vec3::ONE)]
    track_data: Vec3,
    // track_compression_settings: Vec<TrackCompressionSettings>,
}
