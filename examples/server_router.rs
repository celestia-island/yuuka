use anyhow::Result;
use serde::{Deserialize, Serialize};
use yuuka::{auto, derive_struct};

fn main() -> Result<()> {
    derive_struct!(
        #[derive(PartialEq, Serialize, Deserialize)]
        Config {
          port: u16,
          services: [Service {
            domain: Vec<String>,
            rules: [Rule {
              pattern: String,
              method: enum Method {
                Redirect { url: String },
                Proxy { host: String },
                StaticFile { path: String },
                StaticDir { path: String },
              }
            }]
          }]
        }
    );

    let config = auto!(Config {
        port: 8080,
        services: vec![Service {
            domain: vec!["example.com".to_string()],
            rules: vec![
                Rule {
                    pattern: "^/$".to_string(),
                    method: Method::Redirect {
                        url: "https://example.com/index.html".to_string()
                    }
                },
                Rule {
                    pattern: "^/api".to_string(),
                    method: Method::Proxy {
                        host: "http://localhost:8081".to_string()
                    }
                },
                Rule {
                    pattern: "^/static".to_string(),
                    method: Method::StaticDir {
                        path: "/var/www/static".to_string()
                    }
                }
            ]
        }]
    });

    let json_raw = r#"
{
    "port": 8080,
    "services": [
        {
        "domain": ["example.com"],
        "rules": [
            {
            "pattern": "^/$",
            "method": {
                "Redirect": {
                "url": "https://example.com/index.html"
                }
            }
            },
            {
            "pattern": "^/api",
            "method": {
                "Proxy": {
                "host": "http://localhost:8081"
                }
            }
            },
            {
            "pattern": "^/static",
            "method": {
                "StaticDir": {
                "path": "/var/www/static"
                }
            }
            }
        ]
        }
    ]
}
        "#;

    let config_from_json = serde_json::from_str::<Config>(json_raw)?;
    assert_eq!(config, config_from_json);
    Ok(())
}
