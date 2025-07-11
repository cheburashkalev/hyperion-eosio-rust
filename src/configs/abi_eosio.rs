use std::fs;
use std::io::{Write};
use std::sync::OnceLock;
use log::{warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::configs;
use crate::configs::{PATH_CONFIGS_JSON, PATH_WORKDIR};

fn get_def_abi_eosio_json() -> Value {
    json!({
  "version": "eosio::abi/1.1",
  "types": [],
  "structs": [
    {
      "name": "abi_hash",
      "base": "",
      "fields": [
        {
          "name": "owner",
          "type": "name"
        },
        {
          "name": "hash",
          "type": "checksum256"
        }
      ]
    },
    {
      "name": "activate",
      "base": "",
      "fields": [
        {
          "name": "feature_digest",
          "type": "checksum256"
        }
      ]
    },
    {
      "name": "authority",
      "base": "",
      "fields": [
        {
          "name": "threshold",
          "type": "uint32"
        },
        {
          "name": "keys",
          "type": "key_weight[]"
        },
        {
          "name": "accounts",
          "type": "permission_level_weight[]"
        },
        {
          "name": "waits",
          "type": "wait_weight[]"
        }
      ]
    },
    {
      "name": "block_header",
      "base": "",
      "fields": [
        {
          "name": "timestamp",
          "type": "uint32"
        },
        {
          "name": "producer",
          "type": "name"
        },
        {
          "name": "confirmed",
          "type": "uint16"
        },
        {
          "name": "previous",
          "type": "checksum256"
        },
        {
          "name": "transaction_mroot",
          "type": "checksum256"
        },
        {
          "name": "action_mroot",
          "type": "checksum256"
        },
        {
          "name": "schedule_version",
          "type": "uint32"
        },
        {
          "name": "new_producers",
          "type": "producer_schedule?"
        }
      ]
    },
    {
      "name": "blockchain_parameters",
      "base": "",
      "fields": [
        {
          "name": "max_block_net_usage",
          "type": "uint64"
        },
        {
          "name": "target_block_net_usage_pct",
          "type": "uint32"
        },
        {
          "name": "max_transaction_net_usage",
          "type": "uint32"
        },
        {
          "name": "base_per_transaction_net_usage",
          "type": "uint32"
        },
        {
          "name": "net_usage_leeway",
          "type": "uint32"
        },
        {
          "name": "context_free_discount_net_usage_num",
          "type": "uint32"
        },
        {
          "name": "context_free_discount_net_usage_den",
          "type": "uint32"
        },
        {
          "name": "max_block_cpu_usage",
          "type": "uint32"
        },
        {
          "name": "target_block_cpu_usage_pct",
          "type": "uint32"
        },
        {
          "name": "max_transaction_cpu_usage",
          "type": "uint32"
        },
        {
          "name": "min_transaction_cpu_usage",
          "type": "uint32"
        },
        {
          "name": "max_transaction_lifetime",
          "type": "uint32"
        },
        {
          "name": "deferred_trx_expiration_window",
          "type": "uint32"
        },
        {
          "name": "max_transaction_delay",
          "type": "uint32"
        },
        {
          "name": "max_inline_action_size",
          "type": "uint32"
        },
        {
          "name": "max_inline_action_depth",
          "type": "uint16"
        },
        {
          "name": "max_authority_depth",
          "type": "uint16"
        }
      ]
    },
    {
      "name": "canceldelay",
      "base": "",
      "fields": [
        {
          "name": "canceling_auth",
          "type": "permission_level"
        },
        {
          "name": "trx_id",
          "type": "checksum256"
        }
      ]
    },
    {
      "name": "deleteauth",
      "base": "",
      "fields": [
        {
          "name": "account",
          "type": "name"
        },
        {
          "name": "permission",
          "type": "name"
        }
      ]
    },
    {
      "name": "key_weight",
      "base": "",
      "fields": [
        {
          "name": "key",
          "type": "public_key"
        },
        {
          "name": "weight",
          "type": "uint16"
        }
      ]
    },
    {
      "name": "linkauth",
      "base": "",
      "fields": [
        {
          "name": "account",
          "type": "name"
        },
        {
          "name": "code",
          "type": "name"
        },
        {
          "name": "type",
          "type": "name"
        },
        {
          "name": "requirement",
          "type": "name"
        }
      ]
    },
    {
      "name": "newaccount",
      "base": "",
      "fields": [
        {
          "name": "creator",
          "type": "name"
        },
        {
          "name": "name",
          "type": "name"
        },
        {
          "name": "owner",
          "type": "authority"
        },
        {
          "name": "active",
          "type": "authority"
        }
      ]
    },
    {
      "name": "onbilltrx",
      "base": "",
      "fields": [
        {
          "name": "fee_trxs",
          "type": "pay_fee_trx[]"
        }
      ]
    },
    {
      "name": "onblock",
      "base": "",
      "fields": [
        {
          "name": "header",
          "type": "block_header"
        }
      ]
    },
    {
      "name": "onerror",
      "base": "",
      "fields": [
        {
          "name": "sender_id",
          "type": "uint128"
        },
        {
          "name": "sent_trx",
          "type": "bytes"
        }
      ]
    },
    {
      "name": "pay_fee_trx",
      "base": "",
      "fields": [
        {
          "name": "account",
          "type": "name"
        },
        {
          "name": "trx_id",
          "type": "string"
        },
        {
          "name": "cpu_us",
          "type": "uint64"
        },
        {
          "name": "ram_bytes",
          "type": "uint64"
        }
      ]
    },
    {
      "name": "permission_level",
      "base": "",
      "fields": [
        {
          "name": "actor",
          "type": "name"
        },
        {
          "name": "permission",
          "type": "name"
        }
      ]
    },
    {
      "name": "permission_level_weight",
      "base": "",
      "fields": [
        {
          "name": "permission",
          "type": "permission_level"
        },
        {
          "name": "weight",
          "type": "uint16"
        }
      ]
    },
    {
      "name": "producer_key",
      "base": "",
      "fields": [
        {
          "name": "producer_name",
          "type": "name"
        },
        {
          "name": "block_signing_key",
          "type": "public_key"
        }
      ]
    },
    {
      "name": "producer_schedule",
      "base": "",
      "fields": [
        {
          "name": "version",
          "type": "uint32"
        },
        {
          "name": "producers",
          "type": "producer_key[]"
        }
      ]
    },
    {
      "name": "reqactivated",
      "base": "",
      "fields": [
        {
          "name": "feature_digest",
          "type": "checksum256"
        }
      ]
    },
    {
      "name": "reqauth",
      "base": "",
      "fields": [
        {
          "name": "from",
          "type": "name"
        }
      ]
    },
    {
      "name": "setabi",
      "base": "",
      "fields": [
        {
          "name": "account",
          "type": "name"
        },
        {
          "name": "abi",
          "type": "bytes"
        }
      ]
    },
    {
      "name": "setalimits",
      "base": "",
      "fields": [
        {
          "name": "account",
          "type": "name"
        },
        {
          "name": "ram_bytes",
          "type": "int64"
        },
        {
          "name": "net_weight",
          "type": "int64"
        },
        {
          "name": "cpu_weight",
          "type": "int64"
        }
      ]
    },
    {
      "name": "setcode",
      "base": "",
      "fields": [
        {
          "name": "account",
          "type": "name"
        },
        {
          "name": "vmtype",
          "type": "uint8"
        },
        {
          "name": "vmversion",
          "type": "uint8"
        },
        {
          "name": "code",
          "type": "bytes"
        }
      ]
    },
    {
      "name": "setparams",
      "base": "",
      "fields": [
        {
          "name": "params",
          "type": "blockchain_parameters"
        }
      ]
    },
    {
      "name": "setpriv",
      "base": "",
      "fields": [
        {
          "name": "account",
          "type": "name"
        },
        {
          "name": "is_priv",
          "type": "uint8"
        }
      ]
    },
    {
      "name": "setprods",
      "base": "",
      "fields": [
        {
          "name": "schedule",
          "type": "producer_key[]"
        }
      ]
    },
    {
      "name": "unlinkauth",
      "base": "",
      "fields": [
        {
          "name": "account",
          "type": "name"
        },
        {
          "name": "code",
          "type": "name"
        },
        {
          "name": "type",
          "type": "name"
        }
      ]
    },
    {
      "name": "updateauth",
      "base": "",
      "fields": [
        {
          "name": "account",
          "type": "name"
        },
        {
          "name": "permission",
          "type": "name"
        },
        {
          "name": "parent",
          "type": "name"
        },
        {
          "name": "auth",
          "type": "authority"
        }
      ]
    },
    {
      "name": "wait_weight",
      "base": "",
      "fields": [
        {
          "name": "wait_sec",
          "type": "uint32"
        },
        {
          "name": "weight",
          "type": "uint16"
        }
      ]
    }
  ],
  "actions": [
    {
      "name": "activate",
      "type": "activate",
      "ricardian_contract": "---\nspec_version: \"0.2.0\"\ntitle: Activate Protocol Feature\nsummary: 'Activate protocol feature {{nowrap feature_digest}}'\nicon: http://127.0.0.1/ricardian_assets/eosio.contracts/icons/admin.png#9bf1cec664863bd6aaac0f814b235f8799fb02c850e9aa5da34e8a004bd6518e\n---\n\n{{$action.account}} activates the protocol feature with a digest of {{feature_digest}}."
    },
    {
      "name": "canceldelay",
      "type": "canceldelay",
      "ricardian_contract": "---\nspec_version: \"0.2.0\"\ntitle: Cancel Delayed Transaction\nsummary: '{{nowrap canceling_auth.actor}} cancels a delayed transaction'\nicon: http://127.0.0.1/ricardian_assets/eosio.contracts/icons/account.png#3d55a2fc3a5c20b456f5657faf666bc25ffd06f4836c5e8256f741149b0b294f\n---\n\n{{canceling_auth.actor}} cancels the delayed transaction with id {{trx_id}}."
    },
    {
      "name": "deleteauth",
      "type": "deleteauth",
      "ricardian_contract": "---\nspec_version: \"0.2.0\"\ntitle: Delete Account Permission\nsummary: 'Delete the {{nowrap permission}} permission of {{nowrap account}}'\nicon: http://127.0.0.1/ricardian_assets/eosio.contracts/icons/account.png#3d55a2fc3a5c20b456f5657faf666bc25ffd06f4836c5e8256f741149b0b294f\n---\n\nDelete the {{permission}} permission of {{account}}."
    },
    {
      "name": "linkauth",
      "type": "linkauth",
      "ricardian_contract": "---\nspec_version: \"0.2.0\"\ntitle: Link Action to Permission\nsummary: '{{nowrap account}} sets the minimum required permission for the {{#if type}}{{nowrap type}} action of the{{/if}} {{nowrap code}} contract to {{nowrap requirement}}'\nicon: http://127.0.0.1/ricardian_assets/eosio.contracts/icons/account.png#3d55a2fc3a5c20b456f5657faf666bc25ffd06f4836c5e8256f741149b0b294f\n---\n\n{{account}} sets the minimum required permission for the {{#if type}}{{type}} action of the{{/if}} {{code}} contract to {{requirement}}.\n\n{{#if type}}{{else}}Any links explicitly associated to specific actions of {{code}} will take precedence.{{/if}}"
    },
    {
      "name": "newaccount",
      "type": "newaccount",
      "ricardian_contract": "---\nspec_version: \"0.2.0\"\ntitle: Create New Account\nsummary: '{{nowrap creator}} creates a new account with the name {{nowrap name}}'\nicon: http://127.0.0.1/ricardian_assets/eosio.contracts/icons/account.png#3d55a2fc3a5c20b456f5657faf666bc25ffd06f4836c5e8256f741149b0b294f\n---\n\n{{creator}} creates a new account with the name {{name}} and the following permissions:\n\nowner permission with authority:\n{{to_json owner}}\n\nactive permission with authority:\n{{to_json active}}"
    },
    {
      "name": "onbilltrx",
      "type": "onbilltrx",
      "ricardian_contract": ""
    },
    {
      "name": "onblock",
      "type": "onblock",
      "ricardian_contract": ""
    },
    {
      "name": "onerror",
      "type": "onerror",
      "ricardian_contract": ""
    },
    {
      "name": "reqactivated",
      "type": "reqactivated",
      "ricardian_contract": "---\nspec_version: \"0.2.0\"\ntitle: Assert Protocol Feature Activation\nsummary: 'Assert that protocol feature {{nowrap feature_digest}} has been activated'\nicon: http://127.0.0.1/ricardian_assets/eosio.contracts/icons/admin.png#9bf1cec664863bd6aaac0f814b235f8799fb02c850e9aa5da34e8a004bd6518e\n---\n\nAssert that the protocol feature with a digest of {{feature_digest}} has been activated."
    },
    {
      "name": "reqauth",
      "type": "reqauth",
      "ricardian_contract": "---\nspec_version: \"0.2.0\"\ntitle: Assert Authorization\nsummary: 'Assert that authorization by {{nowrap from}} is provided'\nicon: http://127.0.0.1/ricardian_assets/eosio.contracts/icons/account.png#3d55a2fc3a5c20b456f5657faf666bc25ffd06f4836c5e8256f741149b0b294f\n---\n\nAssert that authorization by {{from}} is provided."
    },
    {
      "name": "setabi",
      "type": "setabi",
      "ricardian_contract": "---\nspec_version: \"0.2.0\"\ntitle: Deploy Contract ABI\nsummary: 'Deploy contract ABI on account {{nowrap account}}'\nicon: http://127.0.0.1/ricardian_assets/eosio.contracts/icons/account.png#3d55a2fc3a5c20b456f5657faf666bc25ffd06f4836c5e8256f741149b0b294f\n---\n\nDeploy the ABI file associated with the contract on account {{account}}."
    },
    {
      "name": "setalimits",
      "type": "setalimits",
      "ricardian_contract": "---\nspec_version: \"0.2.0\"\ntitle: Adjust Resource Limits of Account\nsummary: 'Adjust resource limits of account {{nowrap account}}'\nicon: http://127.0.0.1/ricardian_assets/eosio.contracts/icons/admin.png#9bf1cec664863bd6aaac0f814b235f8799fb02c850e9aa5da34e8a004bd6518e\n---\n\n{{$action.account}} updates {{account}}’s resource limits to have a RAM quota of {{ram_bytes}} bytes, a NET bandwidth quota of {{net_weight}} and a CPU bandwidth quota of {{cpu_weight}}."
    },
    {
      "name": "setcode",
      "type": "setcode",
      "ricardian_contract": "---\nspec_version: \"0.2.0\"\ntitle: Deploy Contract Code\nsummary: 'Deploy contract code on account {{nowrap account}}'\nicon: http://127.0.0.1/ricardian_assets/eosio.contracts/icons/account.png#3d55a2fc3a5c20b456f5657faf666bc25ffd06f4836c5e8256f741149b0b294f\n---\n\nDeploy compiled contract code to the account {{account}}."
    },
    {
      "name": "setparams",
      "type": "setparams",
      "ricardian_contract": "---\nspec_version: \"0.2.0\"\ntitle: Set System Parameters\nsummary: 'Set system parameters'\nicon: http://127.0.0.1/ricardian_assets/eosio.contracts/icons/admin.png#9bf1cec664863bd6aaac0f814b235f8799fb02c850e9aa5da34e8a004bd6518e\n---\n\n{{$action.account}} sets system parameters to:\n{{to_json params}}"
    },
    {
      "name": "setpriv",
      "type": "setpriv",
      "ricardian_contract": "---\nspec_version: \"0.2.0\"\ntitle: Make an Account Privileged or Unprivileged\nsummary: '{{#if is_priv}}Make {{nowrap account}} privileged{{else}}Remove privileged status of {{nowrap account}}{{/if}}'\nicon: http://127.0.0.1/ricardian_assets/eosio.contracts/icons/admin.png#9bf1cec664863bd6aaac0f814b235f8799fb02c850e9aa5da34e8a004bd6518e\n---\n\n{{#if is_priv}}\n{{$action.account}} makes {{account}} privileged.\n{{else}}\n{{$action.account}} removes privileged status of {{account}}.\n{{/if}}"
    },
    {
      "name": "setprods",
      "type": "setprods",
      "ricardian_contract": "---\nspec_version: \"0.2.0\"\ntitle: Set Block Producers\nsummary: 'Set block producer schedule'\nicon: http://127.0.0.1/ricardian_assets/eosio.contracts/icons/admin.png#9bf1cec664863bd6aaac0f814b235f8799fb02c850e9aa5da34e8a004bd6518e\n---\n\n{{$action.account}} proposes a block producer schedule of:\n{{#each schedule}}\n  1. {{this.producer_name}} with a block signing key of {{this.block_signing_key}}\n{{/each}}"
    },
    {
      "name": "unlinkauth",
      "type": "unlinkauth",
      "ricardian_contract": "---\nspec_version: \"0.2.0\"\ntitle: Unlink Action from Permission\nsummary: '{{nowrap account}} unsets the minimum required permission for the {{#if type}}{{nowrap type}} action of the{{/if}} {{nowrap code}} contract'\nicon: http://127.0.0.1/ricardian_assets/eosio.contracts/icons/account.png#3d55a2fc3a5c20b456f5657faf666bc25ffd06f4836c5e8256f741149b0b294f\n---\n\n{{account}} removes the association between the {{#if type}}{{type}} action of the{{/if}} {{code}} contract and its minimum required permission.\n\n{{#if type}}{{else}}This will not remove any links explicitly associated to specific actions of {{code}}.{{/if}}"
    },
    {
      "name": "updateauth",
      "type": "updateauth",
      "ricardian_contract": "---\nspec_version: \"0.2.0\"\ntitle: Modify Account Permission\nsummary: 'Add or update the {{nowrap permission}} permission of {{nowrap account}}'\nicon: http://127.0.0.1/ricardian_assets/eosio.contracts/icons/account.png#3d55a2fc3a5c20b456f5657faf666bc25ffd06f4836c5e8256f741149b0b294f\n---\n\nModify, and create if necessary, the {{permission}} permission of {{account}} to have a parent permission of {{parent}} and the following authority:\n{{to_json auth}}"
    }
  ],
  "tables": [
    {
      "name": "abihash",
      "index_type": "i64",
      "key_names": [],
      "key_types": [],
      "type": "abi_hash"
    }
  ],
  "ricardian_clauses": [],
  "error_messages": [],
  "abi_extensions": [],
  "variants": []
})
}
static ABI_EOSIO_CONFIG: OnceLock<String> = OnceLock::new();
const FILE_NAME_ABI_EOSIO_JSON: &str = "abi_eosio.json";

pub fn get_abi_eosio_config() -> &'static String {
    ABI_EOSIO_CONFIG.get_or_init(|| {
        println!("Start loading \'ABI_CONFIG\' file: {}.",FILE_NAME_ABI_EOSIO_JSON);
        let config = configs::load_configs_json(FILE_NAME_ABI_EOSIO_JSON,get_def_abi_eosio_json());
        config.to_string()
    })
}