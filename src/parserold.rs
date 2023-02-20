extern crate nom;
use nom::{
    multi::count,
    bytes::complete::tag,
    combinator::rest,
    number::complete::{le_i32, le_u16, le_u8, le_f32},
    sequence::tuple,
    IResult,
};
use std::fs;

#[derive(Debug, PartialEq)]
pub struct MessageHeader {
    protocol: u8,
    session_id: u8,
    message_type: u16,
    command_type: u8,
    subsystem_number: u8,
    channel: u8,
    sequence_number: u8,
    message_size: i32,
}

struct SystemInformation {
}

struct NavigationOffsets {
    x: f32,
    y: f32,
    latitude: f32,
    longitude: f32,
    aft: f32,
    starboard: f32,
    depth: f32,
    altitude: f32,
    heading: f32,
    pitch: f32,
    roll: f32,
    yaw: f32,
    tow_point_elevation: f32,    
}

fn navigation_offsets(input: &[u8]) -> IResult<&[u8],NavigationOffsets> {
    let mut parser = tuple((
	le_f32, // X offset
	le_f32, // Y offset
	le_f32, // Latitude offset
	le_f32, // Longitude offset
	le_f32, // Aft offset
	le_f32, // Starboard offset
	le_f32, // Depth offset
	le_f32, // Altitude offset	
	le_f32, // Heading offset
	le_f32, // Pitch offset
	le_f32, // Roll offset
	le_f32, // Yaw offset
	le_f32, // Tow point elevation offset
	count(le_f32,3) // Reserved
    ));

    let (input,
	 (
	     x,
	     y,
	     latitude,
	     longitude,
	     aft,
	     starboard,
	     depth,
	     altitude,
	     heading,
	     pitch,
	     roll,
	     yaw,
	     tow_point_elevation,
	     _
	 )) = parser(input)?;

    Ok((input,NavigationOffsets {
		     x,
	     y,
	     latitude,
	     longitude,
	     aft,
	     starboard,
	     depth,
	     altitude,
	     heading,
	     pitch,
	     roll,
	     yaw,
	     tow_point_elevation
    }))
}

struct SonarDataMessage {
}

struct PitchRollData {
}

struct NMEAString {
}

fn message_header(input: &[u8]) -> IResult<&[u8], MessageHeader> {
    let mut parser = tuple((
        tag([0x01, 0x16]), // Start marker
        le_u8,             // Protocol_version
        le_u8,             // Session identifier
        le_u16,            // Message type
        le_u8,             // Command type
        le_u8,             // Subsystem number
        le_u8,             // Channel
        le_u8,             // Sequence number
        le_u16,            // Reserved
        le_i32,            // Message size)
    ));

    let (
        input,
        (
            _,
            protocol,
            session_id,
            message_type,
            command_type,
            subsystem_number,
            channel,
            sequence_number,
            _,
            message_size,
        ),
    ) = parser(input)?;

    Ok((
        input,
        MessageHeader {
            protocol,
            session_id,
            message_type,
            command_type,
            subsystem_number,
            channel,
            sequence_number,
            message_size,
        },
    ))
}

#[test]
fn message_test() {
    let data =
        fs::read("/home/wkearn/Documents/data/PANGAEA/HE501/HE501_Hydro3_025.001.jsf").unwrap();
    let res = message_header(&data).unwrap();
    assert_eq!(res.1.message_size, 13424);
    assert_eq!(res.1.message_type, 182);
    assert_eq!(res.1.protocol, 0x0b);
}
