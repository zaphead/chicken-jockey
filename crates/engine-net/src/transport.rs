use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, Sender, unbounded};
use quinn::{Connection, Endpoint, RecvStream, SendStream};
use tokio::runtime::Runtime;

use crate::cert::{client_config, server_config};
use crate::codec::{
    decode_client_packet, decode_server_packet, encode_client_packet, encode_server_packet,
};
use crate::messages::{ClientPacket, ServerPacket, DEFAULT_PORT};

pub struct NetServer {
    _runtime: Runtime,
    inbound: Receiver<(u32, ClientPacket)>,
    outbound_senders: Arc<Mutex<HashMap<u32, Sender<ServerPacket>>>>,
}

impl NetServer {
    pub fn bind(addr: SocketAddr) -> Self {
        let runtime = Runtime::new().expect("tokio runtime");
        let (inbound_tx, inbound_rx) = unbounded::<(u32, ClientPacket)>();
        let outbound_senders = Arc::new(Mutex::new(HashMap::<u32, Sender<ServerPacket>>::new()));

        let endpoint = {
            let _guard = runtime.enter();
            Endpoint::server(server_config(), addr).expect("bind server")
        };

        let accept_endpoint = endpoint;
        let accept_inbound = inbound_tx;
        let accept_senders = outbound_senders.clone();
        let next_client_id = Arc::new(std::sync::atomic::AtomicU32::new(1));

        runtime.spawn(async move {
            loop {
                let Some(incoming) = accept_endpoint.accept().await else {
                    continue;
                };
                let client_id = next_client_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let inbound = accept_inbound.clone();
                let senders = accept_senders.clone();
                tokio::spawn(async move {
                    match incoming.await {
                        Ok(connection) => {
                            if let Err(error) =
                                handle_server_connection(connection, client_id, inbound, senders).await
                            {
                                log::warn!("server connection {client_id} ended: {error}");
                            }
                        }
                        Err(error) => log::warn!("server accept failed: {error}"),
                    }
                });
            }
        });

        Self {
            _runtime: runtime,
            inbound: inbound_rx,
            outbound_senders,
        }
    }

    pub fn default_addr() -> SocketAddr {
        SocketAddr::from(([127, 0, 0, 1], DEFAULT_PORT))
    }

    pub fn drain_inbound(&self) -> Vec<(u32, ClientPacket)> {
        let mut packets = Vec::new();
        while let Ok(packet) = self.inbound.try_recv() {
            packets.push(packet);
        }
        packets
    }

    pub fn send(&self, client_id: u32, packet: ServerPacket) {
        let senders = self.outbound_senders.lock().expect("outbound senders");
        if let Some(sender) = senders.get(&client_id) {
            let _ = sender.send(packet);
        }
    }

    pub fn client_ids(&self) -> Vec<u32> {
        self.outbound_senders
            .lock()
            .expect("outbound senders")
            .keys()
            .copied()
            .collect()
    }
}

pub struct NetClient {
    _runtime: Runtime,
    outbound: Sender<ClientPacket>,
    inbound: Receiver<ServerPacket>,
    player_id: Arc<Mutex<Option<u32>>>,
}

impl NetClient {
    pub fn connect(addr: SocketAddr) -> Self {
        let runtime = Runtime::new().expect("tokio runtime");
        let (outbound_tx, outbound_rx) = unbounded::<ClientPacket>();
        let (inbound_tx, inbound_rx) = unbounded::<ServerPacket>();
        let player_id = Arc::new(Mutex::new(None));

        let endpoint = {
            let _guard = runtime.enter();
            let mut endpoint = Endpoint::client("0.0.0.0:0".parse().expect("client bind"))
                .expect("client endpoint");
            endpoint.set_default_client_config(client_config());
            endpoint
        };

        let connect_player_id = player_id.clone();
        runtime.spawn(async move {
            match endpoint.connect(addr, "localhost") {
                Ok(connecting) => match connecting.await {
                    Ok(connection) => {
                        if let Err(error) = handle_client_connection(
                            connection,
                            outbound_rx,
                            inbound_tx,
                            connect_player_id,
                        )
                        .await
                        {
                            log::warn!("client connection ended: {error}");
                        }
                    }
                    Err(error) => log::warn!("client connect failed: {error}"),
                },
                Err(error) => log::warn!("client connect setup failed: {error}"),
            }
        });

        Self {
            _runtime: runtime,
            outbound: outbound_tx,
            inbound: inbound_rx,
            player_id,
        }
    }

    pub fn default_addr() -> SocketAddr {
        NetServer::default_addr()
    }

    pub fn drain_inbound(&self) -> Vec<ServerPacket> {
        let mut packets = Vec::new();
        while let Ok(packet) = self.inbound.try_recv() {
            packets.push(packet);
        }
        packets
    }

    pub fn send(&self, packet: ClientPacket) {
        let _ = self.outbound.send(packet);
    }

    pub fn player_id(&self) -> Option<u32> {
        *self.player_id.lock().expect("player id lock")
    }
}

async fn handle_server_connection(
    connection: Connection,
    client_id: u32,
    inbound: Sender<(u32, ClientPacket)>,
    outbound_senders: Arc<Mutex<HashMap<u32, Sender<ServerPacket>>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (mut send, mut recv) = connection.accept_bi().await?;
    send.write_all(&encode_server_packet(&ServerPacket::Welcome { player_id: client_id }))
        .await?;

    let (server_out_tx, server_out_rx) = unbounded::<ServerPacket>();
    outbound_senders
        .lock()
        .expect("outbound senders")
        .insert(client_id, server_out_tx);

    let outbound_task = tokio::spawn(async move {
        while let Ok(packet) = server_out_rx.recv() {
            if write_packet(&mut send, &encode_server_packet(&packet))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let inbound_task = tokio::spawn(async move {
        loop {
            let bytes = match read_packet(&mut recv).await {
                Ok(bytes) => bytes,
                Err(_) => break,
            };
            match decode_client_packet(&bytes) {
                Ok(packet) => {
                    let _ = inbound.send((client_id, packet));
                }
                Err(error) => log::warn!("server decode error: {error:?}"),
            }
        }
    });

    let _ = tokio::join!(outbound_task, inbound_task);
    outbound_senders
        .lock()
        .expect("outbound senders")
        .remove(&client_id);
    Ok(())
}

async fn handle_client_connection(
    connection: Connection,
    outbound_rx: Receiver<ClientPacket>,
    inbound_tx: Sender<ServerPacket>,
    player_id: Arc<Mutex<Option<u32>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (mut send, mut recv) = connection.open_bi().await?;
    send.write_all(&encode_client_packet(&ClientPacket::Join)).await?;

    let welcome = read_packet(&mut recv).await?;
    if let ServerPacket::Welcome { player_id: id } =
        decode_server_packet(&welcome).map_err(|error| format!("welcome decode: {error:?}"))?
    {
        *player_id.lock().expect("player id lock") = Some(id);
        let _ = inbound_tx.send(ServerPacket::Welcome { player_id: id });
    }

    let outbound_task = tokio::spawn(async move {
        while let Ok(packet) = outbound_rx.recv() {
            if write_packet(&mut send, &encode_client_packet(&packet))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let inbound_task = tokio::spawn(async move {
        loop {
            let bytes = match read_packet(&mut recv).await {
                Ok(bytes) => bytes,
                Err(_) => break,
            };
            match decode_server_packet(&bytes) {
                Ok(packet) => {
                    let _ = inbound_tx.send(packet);
                }
                Err(error) => log::warn!("client decode error: {error:?}"),
            }
        }
    });

    let _ = tokio::join!(outbound_task, inbound_task);
    Ok(())
}

async fn read_packet(recv: &mut RecvStream) -> Result<Vec<u8>, quinn::ReadError> {
    let mut len_bytes = [0u8; 4];
    recv.read_exact(&mut len_bytes)
        .await
        .map_err(|error| match error {
            quinn::ReadExactError::FinishedEarly(_) => quinn::ReadError::ClosedStream,
            quinn::ReadExactError::ReadError(read_error) => read_error,
        })?;
    let len = u32::from_le_bytes(len_bytes) as usize;
    let mut bytes = vec![0u8; len];
    recv.read_exact(&mut bytes)
        .await
        .map_err(|error| match error {
            quinn::ReadExactError::FinishedEarly(_) => quinn::ReadError::ClosedStream,
            quinn::ReadExactError::ReadError(read_error) => read_error,
        })?;
    Ok(bytes)
}

async fn write_packet(send: &mut SendStream, payload: &[u8]) -> Result<(), quinn::WriteError> {
    let len = (payload.len() as u32).to_le_bytes();
    send.write_all(&len).await?;
    send.write_all(payload).await?;
    Ok(())
}
