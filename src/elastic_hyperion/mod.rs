use std::error::Error;
use elasticsearch::{Elasticsearch, IndexParts};
use elasticsearch::auth::Credentials::Basic;
use elasticsearch::cert::{Certificate, CertificateValidation};
use elasticsearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use elasticsearch::ilm::IlmPutLifecycleParts;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use crate::{configs};
use crate::configs::elastic_con;
pub async fn create_elastic_client() -> Result<Elasticsearch, Box<dyn Error>> 
{
    let config = elastic_con::get_elastic_con_config();
    let conn_pool = SingleNodeConnectionPool::new(config.url.parse()?);
    let mut http_crt_file = File::open(config.path_cert_validation.as_str()).await?;
    let mut http_buf = Vec::new();
    http_crt_file.read_to_end(&mut http_buf).await?;
    let http_cert = Certificate::from_pem(http_buf.as_slice()).unwrap();
    let transport = TransportBuilder::new(conn_pool)
        .auth(Basic(config.login.clone(),config.pass.clone()))
        .cert_validation(CertificateValidation::Certificate(http_cert))
        .build()?;
    let client = Elasticsearch::new(transport);
    
    client
        .ilm()
        .put_lifecycle(IlmPutLifecycleParts::Policy("hyperion-rollover"))
        .body(configs::ilm_policy::get_ilm_policy_config())
        .send()
        .await?;
    Ok(client)
}