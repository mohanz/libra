// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::vm_error::{StatusType, VMStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Custom JSON RPC server error codes
/// Ranges from -32000 to -32099 - see `https://www.jsonrpc.org/specification#error_object` for details
pub enum ServerCode {
    DefaultServerError = -32000,

    // expose VM status types as server code
    VmValidationError = -32001,
    VmVerificationError = -32002,
    VmInvariantViolationError = -32003,
    VmDeserializationError = -32004,
    VmExecutionError = -32005,
    VmUnknownError = -32006,
    // TODO expose Mempool insertion status as server code
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JsonRpcError {
    pub code: i16,
    pub message: String,
    pub data: Option<Value>,
}

impl std::error::Error for JsonRpcError {}

impl std::fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl JsonRpcError {
    pub(crate) fn serialize(self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    pub(crate) fn invalid_request() -> Self {
        Self {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        }
    }

    pub(crate) fn invalid_params() -> Self {
        Self {
            code: -32602,
            message: "Invalid params".to_string(),
            data: None,
        }
    }

    pub(crate) fn method_not_found() -> Self {
        Self {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }
    }

    pub(crate) fn internal_error(message: String) -> Self {
        Self {
            code: ServerCode::DefaultServerError as i16,
            message: format!("Server error: {}", message),
            data: None,
        }
    }

    pub(crate) fn vm_error(error: VMStatus) -> Self {
        // map VM status to custom server code
        let vm_status_type = error.status_type();
        let code = match vm_status_type {
            StatusType::Validation => ServerCode::VmValidationError,
            StatusType::Verification => ServerCode::VmVerificationError,
            StatusType::InvariantViolation => ServerCode::VmInvariantViolationError,
            StatusType::Deserialization => ServerCode::VmDeserializationError,
            StatusType::Execution => ServerCode::VmExecutionError,
            StatusType::Unknown => ServerCode::VmUnknownError,
        };

        Self {
            code: code as i16,
            message: format!("Server error: VM {} error: {:?}", vm_status_type, error),
            data: Some(serde_json::json!(error)),
        }
    }

    pub fn get_vm_error(&self) -> Option<VMStatus> {
        if let Some(data) = &self.data {
            if let Ok(vm_error) = serde_json::from_value::<VMStatus>(data.clone()) {
                return Some(vm_error);
            }
        }
        None
    }
}
