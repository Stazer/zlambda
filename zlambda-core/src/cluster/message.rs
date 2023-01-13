#[derive(Clone, Debug, Deserialize, Deserialize)]
pub enum ClusterMessage {
    HandshakeRequest(ClusterHandshakeRequestMessage),
    HandshakeResponse(ClusterHandshakeRequestMessage),
}
