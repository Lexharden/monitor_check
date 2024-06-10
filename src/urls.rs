use chrono::{DateTime, Local};
use serde::Serialize;
use std::sync::Arc;
use http_mitm_proxy::MitmProxy;
use http_mitm_proxy::futures::StreamExt;
use tokio::sync::Mutex;

#[derive(Clone, Serialize)]
pub struct UrlRecord {
    pub url: String,
    #[serde(with = "my_date_format")]
    pub time: DateTime<Local>,
}

mod my_date_format {
    use chrono::{DateTime, Local, SecondsFormat};
    use serde::{self, Serializer, Deserializer, Deserialize};

    pub fn serialize<S>(date: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let s = date.to_rfc3339_opts(SecondsFormat::Secs, true);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Local))
    }
}

pub struct UrlHistory {
    records: Arc<Mutex<Vec<UrlRecord>>>,
}

impl UrlHistory {
    pub fn new() -> Self {
        UrlHistory {
            records: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn get_records(&self) -> Vec<UrlRecord> {
        let records = self.records.lock().await;
        records.clone()
    }

    pub async fn start_monitoring(&self) {
        let records = self.records.clone();
        let root_cert = make_root_cert();

        let proxy = MitmProxy::new(
            Some(root_cert),
            tokio_native_tls::native_tls::TlsConnector::new().unwrap(),
        );

        let (mut communications, server) = proxy.bind(("127.0.0.1", 3003)).await.unwrap();
        tokio::spawn(server);

        while let Some(comm) = communications.next().await {
            let uri = comm.request.uri().clone();
            // Save the URL record
            {
                let mut records = records.lock().await;
                records.push(UrlRecord {
                    url: uri.to_string(),
                    time: Local::now(),
                });
            } // MutexGuard is dropped here

            let _ = comm.request_back.send(comm.request);
            if let Ok(Ok(mut response)) = comm.response.await {
                let mut len = 0;
                let body = response.body_mut();
                while let Some(frame) = body.next().await {
                    if let Ok(frame) = frame {
                        if let Some(data) = frame.data_ref() {
                            len += data.len();
                        }
                    }
                }
                println!(
                    "{}\t{}\t{}\t{}",
                    comm.client_addr,
                    uri,
                    response.status(),
                    len
                );
            }
        }
    }
}

fn make_root_cert() -> rcgen::CertifiedKey {
    let mut param = rcgen::CertificateParams::default();

    param.distinguished_name = rcgen::DistinguishedName::new();
    param.distinguished_name.push(
        rcgen::DnType::CommonName,
        rcgen::DnValue::Utf8String("<HTTP-MITM-PROXY CA>".to_string()),
    );
    param.key_usages = vec![
        rcgen::KeyUsagePurpose::KeyCertSign,
        rcgen::KeyUsagePurpose::CrlSign,
    ];
    param.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

    let key_pair = rcgen::KeyPair::generate().unwrap();
    let cert = param.self_signed(&key_pair).unwrap();

    rcgen::CertifiedKey { cert, key_pair }
}
