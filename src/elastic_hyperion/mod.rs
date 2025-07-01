use std::error::Error;
use elasticsearch::{Elasticsearch, IndexParts};
use elasticsearch::auth::Credentials::Basic;
use elasticsearch::cert::{Certificate, CertificateValidation};
use elasticsearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use elasticsearch::ilm::IlmPutLifecycleParts;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use crate::definitions;

pub async fn create_elastic_client() -> Result<Elasticsearch, Box<dyn Error>> 
{
    let conn_pool = SingleNodeConnectionPool::new("https://localhost:9200".parse().unwrap());
    let mut http_crt_file = File::open("/home/andrei/pki/http.crt").await?;
    let mut http_buf = Vec::new();
    http_crt_file.read_to_end(&mut http_buf).await?;
    let http_cert = Certificate::from_pem(http_buf.as_slice()).unwrap();
    let transport = TransportBuilder::new(conn_pool)
        .auth(Basic("elastic".to_string(),"rILpAx=E8ZDhA7S5OF3+".to_string()))
        .cert_validation(CertificateValidation::Certificate(http_cert))
        .build()?;
    let client = Elasticsearch::new(transport);
    
    client
        .ilm()
        .put_lifecycle(IlmPutLifecycleParts::Policy("hyperion-rollover"))
        .body(definitions::get_ilm_policy())
        .send()
        .await?;
    Ok(client)
}