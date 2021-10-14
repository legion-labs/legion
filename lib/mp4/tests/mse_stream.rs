use legion_mp4::MseStreamWriter;

#[test]
fn test_add() {
    use legion_mp4::{AvcConfig, MediaConfig, Mp4Config};
    use std::io::Cursor;
    fn find_nal(stream: &[u8]) -> (&[u8], &[u8]) {
        let mut current = 0;
        let mut start = 0;
        while current < stream.len() - 4 {
            if stream[current] == 0x0
                && stream[current + 1] == 0x0
                && stream[current + 2] == 0x0
                && stream[current + 3] == 0x1
            {
                if start != 0 {
                    return (&stream[start..current], &stream[current..]);
                } else {
                    start = current + 4;
                }
                current += 4;
            } else {
                current += 1;
            }
        }
        (&stream[start..], &[])
    }

    let h264 = &include_bytes!("data/mse.h264")[..];
    let mp4 = &include_bytes!("data/mse.mp4")[..];

    let data = Cursor::new(Vec::<u8>::new());
    let mut mp4_stream = MseStreamWriter::write_start(
        data,
        &Mp4Config {
            major_brand: b"mp42".into(),
            minor_version: 512,
            compatible_brands: vec![b"mp42".into(), b"isom".into()],
            timescale: 1000,
        },
        60,
    )
    .unwrap();
    let (sps, h264) = find_nal(h264);
    let (pps, h264) = find_nal(h264);
    let (idr, _) = find_nal(h264);
    mp4_stream
        .write_index(
            &MediaConfig::AvcConfig(AvcConfig {
                width: 128,
                height: 128,
                seq_param_set: sps.into(),
                pic_param_set: pps.into(),
            })
            .into(),
        )
        .unwrap();
    let mut frame = vec![
        (idr.len() >> 24) as u8,
        ((idr.len() >> 16) & 0xFF) as u8,
        ((idr.len() >> 8) & 0xFF) as u8,
        (idr.len() & 0xFF) as u8,
    ];
    frame.extend_from_slice(idr);
    mp4_stream.write_sample(true, &frame).unwrap();
    let data: Vec<u8> = mp4_stream.into_writer().into_inner();
    assert_eq!(data, mp4);
}
