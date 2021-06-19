use prost::Message;

use std::io::Cursor;

use crate::libs::ndarray::Ndarray;

use super::chunk::Meshes;

#[derive(Debug)]
pub struct ChunkProtocol {
    pub x: i32,
    pub z: i32,
    pub meshes: [Meshes; 1],
    pub voxels: Option<Ndarray<u32>>,
    pub lights: Option<Ndarray<u32>>,
}

#[derive(Debug)]
pub struct PeerProtocol {
    pub id: String,
    pub name: String,
    pub px: f32,
    pub py: f32,
    pub pz: f32,
    pub qx: f32,
    pub qy: f32,
    pub qz: f32,
    pub qw: f32,
}

#[derive(Debug)]
pub struct UpdateProtocol {
    pub vx: i32,
    pub vy: i32,
    pub vz: i32,
    pub r#type: u32,
}

#[derive(Debug)]
pub struct ChatProtocol {
    pub r#type: messages::chat_message::Type,
    pub sender: String,
    pub body: String,
}

pub struct MessageComponents {
    pub r#type: messages::message::Type,
    pub json: Option<String>,
    pub text: Option<String>,
    pub message: Option<ChatProtocol>,
    pub peers: Option<Vec<PeerProtocol>>,
    pub chunks: Option<Vec<ChunkProtocol>>,
    pub updates: Option<Vec<UpdateProtocol>>,
}

impl MessageComponents {
    pub fn default_for(message_type: messages::message::Type) -> Self {
        Self {
            r#type: message_type,
            json: None,
            text: None,
            message: None,
            peers: None,
            chunks: None,
            updates: None,
        }
    }
}

pub mod messages {
    include!(concat!(env!("OUT_DIR"), "/protocol.rs"));
}

impl messages::Message {
    pub fn parse_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.json)
    }
}

pub fn create_message(components: MessageComponents) -> messages::Message {
    let mut message = messages::Message::default();

    message.r#type = components.r#type as i32;

    if let Some(json) = components.json {
        message.json = json;
    }

    if let Some(text) = components.text {
        message.text = text;
    }

    if let Some(chat_message) = components.message {
        message.message = Some(messages::ChatMessage {
            r#type: chat_message.r#type as i32,
            body: chat_message.body,
            sender: chat_message.sender,
        });
    }

    if let Some(peers) = components.peers {
        message.peers = peers
            .into_iter()
            .map(|peer| messages::Peer {
                id: peer.id,
                name: peer.name,
                px: peer.px,
                py: peer.py,
                pz: peer.pz,
                qx: peer.qx,
                qy: peer.qy,
                qz: peer.qz,
                qw: peer.qw,
            })
            .collect()
    }

    if let Some(chunks) = components.chunks {
        message.chunks = chunks
            .into_iter()
            .map(|chunk| messages::Chunk {
                meshes: chunk
                    .meshes
                    .into_iter()
                    .map(|mesh| {
                        let opaque = mesh.opaque.as_ref();
                        let transparent = mesh.transparent.as_ref();

                        messages::Mesh {
                            opaque: if let Some(opaque) = opaque {
                                Some(messages::Geometry {
                                    aos: opaque.aos.to_owned(),
                                    indices: opaque.indices.to_owned(),
                                    positions: opaque.positions.to_owned(),
                                    sunlights: opaque.sunlights.to_owned(),
                                    torch_lights: opaque.torch_lights.to_owned(),
                                    uvs: opaque.uvs.to_owned(),
                                })
                            } else {
                                None
                            },
                            transparent: if let Some(transparent) = transparent {
                                Some(messages::Geometry {
                                    aos: transparent.aos.to_owned(),
                                    indices: transparent.indices.to_owned(),
                                    positions: transparent.positions.to_owned(),
                                    sunlights: transparent.sunlights.to_owned(),
                                    torch_lights: transparent.torch_lights.to_owned(),
                                    uvs: transparent.uvs.to_owned(),
                                })
                            } else {
                                None
                            },
                        }
                    })
                    .collect(),
                lights: if let Some(l) = chunk.lights {
                    l.data
                } else {
                    Vec::<u32>::new()
                },
                voxels: if let Some(v) = chunk.voxels {
                    v.data
                } else {
                    Vec::<u32>::new()
                },
                x: chunk.x,
                z: chunk.z,
            })
            .collect()
    }

    if let Some(updates) = components.updates {
        message.updates = updates
            .into_iter()
            .map(|update| messages::Update {
                r#type: update.r#type,
                vx: update.vx,
                vy: update.vy,
                vz: update.vz,
            })
            .collect()
    }

    message
}

pub fn encode_message(message: &messages::Message) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.reserve(message.encoded_len());
    message.encode(&mut buf).unwrap();
    buf
}

pub fn decode_message(buf: &[u8]) -> Result<messages::Message, prost::DecodeError> {
    messages::Message::decode(&mut Cursor::new(buf))
}
