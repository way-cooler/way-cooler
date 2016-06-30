//! Test the command channel's integrity/ability to run.

use super::super::{channel, command};

const WRITE_ERR: &'static str = "Unable to write to channel!";

fn ipc_input() -> (Vec<u8>, u32) {
    let mut turn = Vec::new();
    let mut count = 0u32;

    channel::write_packet(&mut turn, &json!({
        "type" => ("version".to_json())
    })).expect(WRITE_ERR); count += 1;
    channel::write_packet(&mut turn, &json!({
        "type" => ("commands".to_json())
    })).expect(WRITE_ERR); count += 1;
    channel::write_packet(&mut turn, &json!({
        "type" => ("get".to_json()),
        "key" => ("point".to_json())
    })).expect(WRITE_ERR); count += 1;
    channel::write_packet(&mut turn, &json!({
        "type" => ("run".to_json()),
        "key" => ("command".to_json())
    })).expect(WRITE_ERR); count += 1;
    channel::write_packet(&mut turn, &json!({
        "type" => ("set".to_json()),
        "key" => "bool",
        "value" => false
    })).expect(WRITE_ERR); count += 1;
    channel::write_packet(&mut turn, &json!({
        "type" => "set",
        "key" => "set_prop",
        "value" => 32u64
    })).expect(WRITE_ERR); count += 1;
    channel::write_packet(&mut turn, &json!({
        "type" => "exists",
        "key" => "null"
    })).expect(WRITE_ERR); count += 1;
    channel::write_packet(&mut turn, &json!({
        "type" => "run",
        "key" => "command"
    })).expect(WRITE_ERR); count += 1;
    channel::write_packet(&mut turn, &json!({
        "type" => "get",
        "key" => "get_prop"
    })).expect(WRITE_ERR); count += 1;
    (turn, count)
}

fn ipc_output() -> (Vec<u8>, u32) {
    let mut turn = Vec::new();
    let mut count = 0u32;

    channel::write_packet(&mut turn, &json!({
        "type" => "success",
        "value" => (super::super::VERSION)
    })).expect(WRITE_ERR); count += 1;
    channel::write_packet(&mut turn, &json!({
        "type" => "success",
        "value" => (json!([ "get", "set", "commands", "run", "version"]))
    })).expect(WRITE_ERR);
    channel::write_packet(&mut turn, &json!({
        "type" => "get",
        "key" => "point",
        "value" => (json!({
            "x" => 10, "y" => 12
        }))
    })).expect(WRITE_ERR); count += 1;
    channel::write_packet(&mut turn, &json!({
        "type" => ("success".to_json())
    })).expect(WRITE_ERR); count += 1;
    channel::write_packet(&mut turn, &json!({
        "type" => ("success".to_json())
    })).expect(WRITE_ERR); count += 1;
    channel::write_packet(&mut turn, &json!({
        "type" => "success"
    })).expect(WRITE_ERR); count += 1;
    channel::write_packet(&mut turn, &json!({
        "type" => "success",
        "key_type" => "Object",
        "flags" => (json!([ "read".to_json(), "write".to_json() ]))
    })).expect(WRITE_ERR); count += 1;
    channel::write_packet(&mut turn, &json!({
        "type" => "run",
        "key" => "command"
    })).expect(WRITE_ERR); count += 1;
    channel::write_packet(&mut turn, &json!({
        "type" => "success",
        "value" => 12
    })).expect(WRITE_ERR); count += 1;
    (turn, count)
}

use std::io::{Read, Write};
use std::io::Result as IOResult;

struct BidirectionalStream {
    pub input: Vec<u8>,
    pub output: Vec<u8>
}

impl BidirectionalStream {
    fn new(input: Vec<u8>, output: Vec<u8>) -> BidirectionalStream {
        BidirectionalStream { input: input, output: output }
    }
}

impl Read for BidirectionalStream {
    fn read(&mut self, buf: &mut [u8]) -> IOResult<usize> {
        let result = (&*self.input).read(buf);
        // If this weren't for testing purposes I'd use slices and stuff
        if !self.input.is_empty() {
            for _ in 0 .. buf.len() {
            self.input.remove(0);
            }
        }
        result
    }
}

impl Write for BidirectionalStream {
    fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
        self.output.write(buf)
    }

    fn flush(&mut self) -> IOResult<()> {
        self.output.flush()
    }
}

#[test]
fn command_loop() {
    let (mut in_channel, mut _in_len) = ipc_input();
    let (mut expected_out, mut _out_len) = ipc_output();

    let mut thread_out = Vec::new();
    let mut stream = BidirectionalStream::new(in_channel, thread_out);

    command::thread(&mut stream);

    assert_eq!(stream.output, expected_out);
}
fn read_after_take() {
}
