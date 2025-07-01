use serde_json::{json, Value};

/// Функция, возвращающая твою политику как JSON объект
pub fn get_ilm_policy() -> Value {

    json!({
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
        })
}