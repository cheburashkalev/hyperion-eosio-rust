use std::error::Error;
use std::sync::OnceLock;
use elasticsearch::{Elasticsearch, IndexParts};
use elasticsearch::auth::Credentials::Basic;
use elasticsearch::cert::{Certificate, CertificateValidation};
use elasticsearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use elasticsearch::ilm::IlmPutLifecycleParts;
use elasticsearch::indices::IndicesPutTemplateParts;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use crate::{configs};
use crate::configs::{elastic_con, templates};
use crate::configs::ship::ShipConConfig;

static SHIP_CON_CONFIG: OnceLock<Elasticsearch> = OnceLock::new();
pub async fn get_elastic_client() -> Result<&'static Elasticsearch, Box<dyn Error>>
{
    let config = elastic_con::get_elastic_con_config();
    let mut http_crt_file = File::open(config.path_cert_validation.as_str()).await.unwrap();
    let mut http_buf = Vec::new();
    http_crt_file.read_to_end(&mut http_buf).await?;
    let client = SHIP_CON_CONFIG.get_or_init(|| {
        let conn_pool = SingleNodeConnectionPool::new(config.url.parse().unwrap());
        let http_cert = Certificate::from_pem(http_buf.as_slice()).unwrap();
        let transport = TransportBuilder::new(conn_pool)
            .auth(Basic(config.login.clone(), config.pass.clone()))
            .cert_validation(CertificateValidation::Certificate(http_cert))
            .build().unwrap();
        Elasticsearch::new(transport)
    });
    client
        .ilm()
        .put_lifecycle(IlmPutLifecycleParts::Policy("hyperion-rollover"))
        .body(configs::ilm_policy::get_ilm_policy_config())
        .send()
        .await?;
    client
        .indices()
        .put_template(IndicesPutTemplateParts::Name("gf-abi"))
        .body(templates::abi())
        .send()
        .await?;
    Ok(client)
}