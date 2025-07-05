use crate::configs;
use crate::configs::ship::ShipConConfig;
use crate::configs::{elastic_con, templates};
use elasticsearch::auth::Credentials::Basic;
use elasticsearch::cert::{Certificate, CertificateValidation};
use elasticsearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use elasticsearch::ilm::{IlmDeleteLifecycleParts, IlmPutLifecycleParts};
use elasticsearch::indices::IndicesPutTemplateParts;
use elasticsearch::{Elasticsearch, IndexParts};
use std::error::Error;
use std::sync::OnceLock;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

static ELASTIC_CON: OnceLock<Elasticsearch> = OnceLock::new();
pub async fn get_elastic_client() -> Result<&'static Elasticsearch, Box<dyn Error>> {
    let config = elastic_con::get_elastic_con_config();
    let mut http_crt_file = File::open(config.path_cert_validation.as_str())
        .await
        .unwrap();
    let mut http_buf = Vec::new();
    http_crt_file.read_to_end(&mut http_buf).await?;
    let elas = ELASTIC_CON.get();
    if elas.is_some() {
        Ok(elas.unwrap())
    } else {
        let client = ELASTIC_CON.get_or_init(|| {
            let conn_pool = SingleNodeConnectionPool::new(config.url.parse().unwrap());
            let http_cert = Certificate::from_pem(http_buf.as_slice()).unwrap();
            let transport = TransportBuilder::new(conn_pool)
                .auth(Basic(config.login.clone(), config.pass.clone()))
                .cert_validation(CertificateValidation::Certificate(http_cert))
                .build()
                .unwrap();
            Elasticsearch::new(transport)
        });

        for config in configs::ilm_policy::get_ilm_policy_config()
            .as_array()
            .unwrap()
        {
            let name_policy = config["policy"].as_str().unwrap();
            let body = &config["body"];
            client
                .ilm()
                .delete_lifecycle(IlmDeleteLifecycleParts::Policy(name_policy))
                .send()
                .await?;
            let r = client
                .ilm()
                .put_lifecycle(IlmPutLifecycleParts::Policy(name_policy))
                .body(body)
                .send()
                .await?
                .error_for_status_code();
            match r {
                Ok(r) => {}
                Err(e) => panic!("{:?}", e),
            }
        }
        for (name, template) in &templates::templates() {
            let r = client
                .indices()
                .put_template(IndicesPutTemplateParts::Name(name))
                .body(template)
                .send()
                .await?
                .error_for_status_code();
            match r {
                Ok(r) => {}
                Err(e) => panic!("{:?}", e),
            }
        }

        Ok(client)
    }
}
