use crate::configs;
use serde_json::{Value, json};
use std::sync::OnceLock;

fn get_def_ilm_policy_json() -> Value {
    json!([{
        "policy": "hyperion-rollover",
        "body": {
            "policy": {
                "phases": {
                    "hot": {
                        "min_age": "0ms",
                        "actions": {
                            "rollover": {
                                "max_size": "200gb",
                                "max_age": "60d",
                                "max_docs": 100000000
                            },
                            "set_priority": {
                                "priority": 50
                            }
                        }
                    },
                    "warm": {
                        "min_age": "2d",
                        "actions": {
                            "allocate": {
                                "exclude": {
                                    "data": "hot"
                                }
                            },
                            "set_priority": {
                                "priority": 25
                            }
                        }
                    }
                }
            }
        }
    },
    {
        "policy": "10G30D",
        "body": {
            "policy": {
                "phases": {
                    "hot": {
                        "min_age": "0ms",
                        "actions": {
                            "rollover": {
                                "max_age": "30d",
                                "max_size": "10gb",
                                "max_docs": 100000000
                            },
                            "set_priority": {
                                "priority": 100
                            }
                        }
                    }
                }
            }
        }
    },
    {
        "policy": "200G",
        "body": {
            "policy": {
                "phases": {
                    "hot": {
                        "min_age": "0ms",
                        "actions": {
                            "rollover": {
                                "max_size": "200gb"
                            },
                            "set_priority": {
                                "priority": 100
                            }
                        }
                    }
                }
            }
        }
    }])
}
static ILM_POLICY_CONFIG: OnceLock<Value> = OnceLock::new();
const FILE_NAME_ILM_POLICY_JSON: &str = "elastic_ilm_policy.json";
pub fn get_ilm_policy_config() -> &'static Value {
    ILM_POLICY_CONFIG.get_or_init(|| {
        println!(
            "Start loading \'ILM_POLICY\' file: {}.",
            FILE_NAME_ILM_POLICY_JSON
        );
        configs::load_configs_json(FILE_NAME_ILM_POLICY_JSON, get_def_ilm_policy_json())
    })
}
