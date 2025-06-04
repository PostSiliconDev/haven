use anyhow::Result;
use futures::future::select_all;
use haven_db::Database;
use hickory_proto::op::Message;
use hickory_proto::rr::{rdata, Name, RData, Record, RecordType};
use hickory_proto::serialize::binary::BinEncodable;
use reqwest::Client;
use sqlx::Row;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

pub struct QueryHandler {
    udp_upstreams: Vec<SocketAddr>,
    doh_upstreams: Vec<String>,
    client: Client,
}

impl QueryHandler {
    pub fn new(udp_upstreams: Vec<SocketAddr>, doh_upstreams: Vec<String>) -> Self {
        Self {
            udp_upstreams,
            doh_upstreams,
            client: Client::new(),
        }
    }

    pub async fn query_local(
        &self,
        request: Message,
        database: &Database,
    ) -> Result<Option<Message>> {
        let query = request
            .queries()
            .first()
            .ok_or(anyhow::anyhow!("No query found"))?;
        let domain = query.name().to_string();
        let record_type = query.query_type();

        // Only support A, AAAA, and CNAME records
        if !matches!(
            record_type,
            RecordType::A | RecordType::AAAA | RecordType::CNAME
        ) {
            return Ok(None);
        }

        // Query the database
        let record = sqlx::query("SELECT record, ttl FROM record WHERE domain = $1 AND type = $2")
            .bind(&domain)
            .bind(record_type.to_string())
            .fetch_optional(database.pool())
            .await?;

        let (record, ttl) = if let Some(row) = record {
            (
                row.get::<String, _>("record"),
                row.get::<Option<i32>, _>("ttl").unwrap_or_default(),
            )
        } else {
            return Ok(None);
        };

        // Create response message
        let mut response = Message::new();
        response.set_id(request.id());
        response.set_message_type(hickory_proto::op::MessageType::Response);
        response.set_op_code(request.op_code());
        response.set_response_code(hickory_proto::op::ResponseCode::NoError);
        response.add_query(query.clone());

        // Create DNS record based on type and value
        let name = query.name().clone();

        // Set the record data based on type
        let data = match record_type {
            RecordType::A => {
                if let Ok(ip) = record.parse::<Ipv4Addr>() {
                    RData::A(rdata::A::from(ip))
                } else {
                    return Ok(None);
                }
            }
            RecordType::AAAA => {
                if let Ok(ip) = record.parse::<Ipv6Addr>() {
                    RData::AAAA(rdata::AAAA::from(ip))
                } else {
                    return Ok(None);
                }
            }
            RecordType::CNAME => {
                if let Ok(name) = Name::from_utf8(&record) {
                    RData::CNAME(rdata::CNAME(name))
                } else {
                    return Ok(None);
                }
            }
            _ => return Ok(None),
        };

        let dns_record = Record::from_rdata(name, ttl as u32, data);

        response.add_answer(dns_record);

        log::debug!("Response: {:?}", response);

        Ok(Some(response))
    }

    pub async fn query_upstreams(&self, request: Message) -> Result<Vec<u8>> {
        // Create futures for UDP requests
        let mut udp_futures = Vec::new();
        for upstream in &self.udp_upstreams {
            let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
            let request = request.clone();
            let upstream = *upstream;
            udp_futures.push(tokio::spawn(async move {
                let mut response_buf = [0u8; 512];
                socket.send_to(&request.to_bytes()?, upstream).await?;
                let (size, _) = socket.recv_from(&mut response_buf).await?;
                Ok::<_, anyhow::Error>(response_buf[..size].to_vec())
            }));
        }

        // Create futures for DoH requests
        let mut doh_futures = Vec::new();
        for upstream in &self.doh_upstreams {
            let client = self.client.clone();
            let request = request.clone();
            let upstream = upstream.clone();
            doh_futures.push(tokio::spawn(async move {
                let response = client
                    .post(upstream)
                    .body(request.to_bytes()?)
                    .send()
                    .await?
                    .bytes()
                    .await?;
                Ok::<_, anyhow::Error>(response.to_vec())
            }));
        }

        // Combine all futures
        let mut all_futures = Vec::new();
        all_futures.extend(udp_futures);
        all_futures.extend(doh_futures);

        let (result, _, _) = select_all(all_futures).await;

        let res = result??;

        Ok(res)
    }
}
