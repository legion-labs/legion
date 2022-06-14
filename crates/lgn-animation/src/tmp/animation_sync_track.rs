use crate::tmp::animation_event::Event;

pub struct SyncTrackTime {
    event_idx: i32,
    percentage_through: f32,
}

pub struct SyncTrack {
    sync_events: Vec<Event>,
    start_event_offset: i32,
}

impl SyncTrack {
    #[inline]
    fn calculate_duration_synchronized(/* */) {
        /* */
    }
    #[inline]
    fn get_num_events() {
        /* */
    }
    #[inline]
    fn has_start_offset() {
        /* */
    }
    #[inline]
    fn get_event() {
        /* */
    }
    #[inline]
    fn get_event_without_offset() {
        /* */
    }
    #[inline]
    fn get_event_duration() { // Return value is a percentage
                              /* */
    }
    #[inline]
    fn get_event_duration_without_offset() { // Return value is a percentage
                                             /* */
    }
    #[inline]
    fn update_event_time() {
        /* */
    }
    #[inline]
    fn update_event_position_without_offset() {
        /* */
    }
    #[inline]
    fn get_start_time() {
        /* */
    }
    #[inline]
    fn get_end_time() {
        /* */
    }
    #[inline]
    fn get_offset_start_time() {
        /* */
    }
    #[inline]
    fn get_offset_end_time() {
        /* */
    }
    fn calculate_percentage_covered() {} // Return value is a percentage

    #[inline]
    fn clamp_index_to_track() {
        /* */
    }
}
