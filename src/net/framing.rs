use anyhow::{bail, Context, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Encodes a 64-bit integer into a Protobuf varint byte vector
pub fn encode_varint(mut value: u64, buf: &mut Vec<u8>) {
    while value >= 0x80 {
        buf.push((value as u8 & 0x7F) | 0x80);
        value >>= 7;
    }
    buf.push(value as u8);
}

/// Reads a length-delimited protobuf message from an async stream.
/// The length delimiter is a standard Protobuf varint.
pub async fn read_delimited_message<R, M>(reader: &mut R) -> Result<M>
where
    R: AsyncRead + Unpin,
    M: prost::Message + Default,
{
    let mut length: usize = 0;
    let mut shift: u32 = 0;
    loop {
        let byte = reader.read_u8().await.context("Failed reading varint byte")?;
        length |= ((byte & 0x7F) as usize) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            bail!("Varint length overflowed 64 bits");
        }
    }

    if length > 16 * 1024 * 1024 {
        bail!("Protobuf message exceeds safe limit: {} bytes", length);
    }

    let mut buf = vec![0u8; length];
    reader
        .read_exact(&mut buf)
        .await
        .context("Failed reading protobuf payload bytes")?;

    let msg = M::decode(buf.as_slice()).context("Failed decoding protobuf message")?;
    Ok(msg)
}

/// Writes a length-delimited protobuf message to an async stream.
/// Prepends the length encoded as a standard Protobuf varint.
pub async fn write_delimited_message<W, M>(writer: &mut W, msg: &M) -> Result<()>
where
    W: AsyncWrite + Unpin,
    M: prost::Message,
{
    let mut buf = Vec::with_capacity(msg.encoded_len() + 8);
    encode_varint(msg.encoded_len() as u64, &mut buf);
    msg.encode(&mut buf).context("Failed encoding protobuf message")?;
    writer
        .write_all(&buf)
        .await
        .context("Failed writing protobuf message")?;
    writer.flush().await.context("Failed flushing stream")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_varint_encoding() {
        let mut buf = Vec::new();
        encode_varint(0, &mut buf);
        assert_eq!(buf, vec![0]);

        buf.clear();
        encode_varint(1, &mut buf);
        assert_eq!(buf, vec![1]);

        buf.clear();
        encode_varint(127, &mut buf);
        assert_eq!(buf, vec![127]);

        buf.clear();
        encode_varint(128, &mut buf);
        assert_eq!(buf, vec![0x80, 0x01]);

        buf.clear();
        encode_varint(300, &mut buf);
        assert_eq!(buf, vec![0xAC, 0x02]);
    }

    #[tokio::test]
    async fn test_read_write_delimited() {
        use crate::proto::polo::OuterMessage;

        let msg = OuterMessage {
            protocol_version: 2,
            status: 200,
            ..Default::default()
        };

        let mut buffer = Vec::new();
        write_delimited_message(&mut buffer, &msg).await.unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded: OuterMessage = read_delimited_message(&mut cursor).await.unwrap();

        assert_eq!(decoded.protocol_version, 2);
        assert_eq!(decoded.status, 200);
    }
}

