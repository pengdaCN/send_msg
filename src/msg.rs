use async_trait::async_trait;
use libp2p::core::upgrade;
use libp2p::futures::{AsyncRead, AsyncWrite};
use libp2p::request_response::codec;

#[derive(Clone)]
pub struct MsgProto;

impl codec::ProtocolName for MsgProto {
    fn protocol_name(&self) -> &[u8] {
        b"/send_msg/1.0"
    }
}

#[derive(Debug)]
pub struct Message(pub Vec<u8>);

#[derive(Debug, Clone)]
pub struct Codec;

impl Codec {
    const MAX_LENGTH: usize = 2 << 20;

    async fn read_message<T>(io: &mut T) -> std::io::Result<Message>
    where
        T: AsyncRead + Unpin + Send,
    {
        let x = upgrade::read_length_prefixed(io, Self::MAX_LENGTH).await?;

        Ok(Message(x))
    }

    async fn write_message<T>(io: &mut T, msg: &Message) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        upgrade::write_length_prefixed(io, &msg.0).await
    }
}

#[async_trait]
impl codec::RequestResponseCodec for Codec {
    type Protocol = MsgProto;
    type Request = Message;
    type Response = Message;

    async fn read_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        Self::read_message(io).await
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        Self::read_message(io).await
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        Self::write_message(io, &req).await
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        Self::write_message(io, &res).await
    }
}
