use anyhow::Result;
use futures::future::select_all;
use haven_db::Database;
use hickory_proto::op::{Header, Message, Query};
use hickory_proto::rr::{rdata, Name, RData, Record, RecordType};
use hickory_proto::serialize::binary::{BinDecodable, BinEncodable};
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
        query: Query,
        database: &Database,
    ) -> Result<Option<Record<RData>>> {
        let domain = query.name().to_string();
        let record_type = query.query_type();

        let mut result_record = None;

        // Only support A, AAAA, and CNAME records
        if !matches!(
            record_type,
            RecordType::A | RecordType::AAAA | RecordType::CNAME
        ) {
            return Ok(result_record);
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
            return Ok(result_record);
        };

        // Create DNS record based on type and value
        let name = query.name().clone();

        // Set the record data based on type
        match record_type {
            RecordType::A => {
                if let Ok(ip) = record.parse::<Ipv4Addr>() {
                    let dns_record =
                        Record::from_rdata(name, ttl as u32, RData::A(rdata::A::from(ip)));
                    result_record = Some(dns_record);
                }
            }
            RecordType::AAAA => {
                if let Ok(ip) = record.parse::<Ipv6Addr>() {
                    let dns_record =
                        Record::from_rdata(name, ttl as u32, RData::AAAA(rdata::AAAA::from(ip)));
                    result_record = Some(dns_record);
                }
            }
            RecordType::CNAME => {
                if let Ok(name) = Name::from_utf8(&record) {
                    let dns_record = Record::from_rdata(
                        name.clone(),
                        ttl as u32,
                        RData::CNAME(rdata::CNAME(name.clone())),
                    );
                    result_record = Some(dns_record);
                }
            }
            _ => {}
        };

        Ok(result_record)
    }

    pub async fn query_upstreams(
        &self,
        header: &Header,
        query: &Query,
    ) -> Result<Vec<Record<RData>>> {
        let mut request = Message::new();
        request.set_header(header.clone());
        request.add_query(query.clone());

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

        let response = Message::from_bytes(&res)?;

        Ok(response.answers().to_vec())
    }

    pub async fn query_cache(
        &self,
        _query: &Query,
        _database: &Database,
    ) -> Result<Option<Record<RData>>> {
        Ok(None)
    }

    pub async fn do_query(
        &self,
        header: &Header,
        query: &Query,
        database: &Database,
    ) -> Result<Vec<Record<RData>>> {
        let mut records = vec![];

        let query_type = query.query_type();
        let name = query.name();
        // if matches!(record_type, RecordType::A | RecordType::AAAA) {}

        log::debug!("Query cache: {} {}", query_type, name);
        if let Some(record) = self.query_cache(query, database).await? {
            records.push(record);
            return Ok(records);
        }

        log::debug!("Cache miss, Query local: {} {}", query_type, name);
        if let Some(record) = self.query_local(query.clone(), database).await? {
            records.push(record);
            return Ok(records);
        }

        log::debug!("Cache miss, Query upstreams: {} {}", query_type, name);
        let upstream_records = self.query_upstreams(header, query).await?;
        records.extend(upstream_records);

        Ok(records)
    }
}
