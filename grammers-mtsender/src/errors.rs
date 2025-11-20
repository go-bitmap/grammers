// Copyright 2020 - developers of the `grammers` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use grammers_mtproto::{authentication, mtp, transport};
use grammers_tl_types as tl;
use std::{fmt, io};

/// This error occurs when reading from the network fails.
#[derive(Debug)]
pub enum ReadError {
    /// Standard I/O error.
    Io(io::Error),
    /// Error propagated from the underlying [`transport`].
    Transport(transport::Error),
    /// Error propagated from attempting to deserialize an invalid [`tl::Deserializable`].
    Deserialize(mtp::DeserializeError),
}

impl std::error::Error for ReadError {}

impl Clone for ReadError {
    fn clone(&self) -> Self {
        match self {
            Self::Io(e) => {
                // 安全地克隆 io::Error，避免访问可能无效的内部状态
                // 优先使用 raw_os_error 重建错误，这是最安全的方式
                if let Some(os_err) = e.raw_os_error() {
                    Self::Io(io::Error::from_raw_os_error(os_err))
                } else {
                    // 如果无法获取 raw_os_error，使用错误类型创建新的错误
                    // 避免调用可能访问无效内存的 to_string() 方法
                    // 使用错误类型的字符串表示作为消息
                    let kind = e.kind();
                    Self::Io(io::Error::new(
                        kind,
                        format!("IO error ({:?})", kind),
                    ))
                }
            }
            Self::Transport(e) => Self::Transport(e.clone()),
            Self::Deserialize(e) => Self::Deserialize(e.clone()),
        }
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => {
                // 安全地格式化 io::Error，避免访问可能无效的内部状态
                // 优先使用 raw_os_error，如果不可用则使用错误类型
                if let Some(os_err) = err.raw_os_error() {
                    write!(f, "read error, IO failed: OS error {}", os_err)
                } else {
                    write!(f, "read error, IO failed: {:?}", err.kind())
                }
            }
            Self::Transport(err) => write!(f, "read error, transport-level: {err}"),
            Self::Deserialize(err) => write!(f, "read error, bad response: {err}"),
        }
    }
}

impl From<io::Error> for ReadError {
    fn from(error: io::Error) -> Self {
        // 安全地转换 io::Error，立即进行安全克隆以避免后续访问无效内存
        // 优先使用 raw_os_error 重建错误，这是最安全的方式
        if let Some(os_err) = error.raw_os_error() {
            Self::Io(io::Error::from_raw_os_error(os_err))
        } else {
            // 如果无法获取 raw_os_error，使用错误类型创建新的错误
            // 避免直接包装可能无效的 io::Error
            let kind = error.kind();
            Self::Io(io::Error::new(
                kind,
                format!("IO error ({:?})", kind),
            ))
        }
    }
}

impl From<transport::Error> for ReadError {
    fn from(error: transport::Error) -> Self {
        Self::Transport(error)
    }
}

impl From<mtp::DeserializeError> for ReadError {
    fn from(error: mtp::DeserializeError) -> Self {
        Self::Deserialize(error)
    }
}

impl From<tl::deserialize::Error> for ReadError {
    fn from(error: tl::deserialize::Error) -> Self {
        Self::Deserialize(error.into())
    }
}

/// The error type reported by the server when a request is misused.
///
/// These are returned when Telegram respond to an RPC with [`tl::types::RpcError`].
#[derive(Clone, Debug, PartialEq)]
pub struct RpcError {
    /// A numerical value similar to [HTTP response status codes](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Status).
    pub code: i32,

    /// The ASCII error name, normally in screaming snake case.
    ///
    /// Digit words are removed from the name and put in the [`RpcError::value`] instead.
    /// ```
    /// use grammers_mtsender::RpcError;
    /// let rpc_error = RpcError::from(grammers_tl_types::types::RpcError {
    ///         error_code: 500, error_message: "INTERDC_2_CALL_ERROR".into() });
    /// assert_eq!(rpc_error.name, "INTERDC_CALL_ERROR");
    /// assert_eq!(rpc_error.value, Some(2));
    /// ```
    pub name: String,

    /// If the error contained an additional integer value, it will be present here and removed from the [`RpcError::name`].
    /// ```
    /// use grammers_mtsender::RpcError;
    /// let rpc_error = RpcError::from(grammers_tl_types::types::RpcError {
    ///         error_code: 420, error_message: "FLOOD_WAIT_31".into() });
    /// assert_eq!(rpc_error.name, "FLOOD_WAIT");
    /// assert_eq!(rpc_error.value, Some(31));
    /// ```
    pub value: Option<u32>,

    /// The constructor identifier of the request that triggered this error.
    /// Won't be present if the error was artificially constructed.
    pub caused_by: Option<u32>,
}

impl std::error::Error for RpcError {}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rpc error {}: {}", self.code, self.name)?;
        if let Some(caused_by) = self.caused_by {
            write!(f, " caused by {}", tl::name_for_id(caused_by))?;
        }
        if let Some(value) = self.value {
            write!(f, " (value: {value})")?;
        }
        Ok(())
    }
}

impl From<tl::types::RpcError> for RpcError {
    fn from(error: tl::types::RpcError) -> Self {
        // Extract the numeric value in the error, if any
        if let Some((value, parsed_value)) = error
            .error_message
            .split(|c: char| !c.is_ascii_digit())
            .flat_map(|value| {
                value
                    .parse::<u32>()
                    .map(|parsed_value| (value, parsed_value))
            })
            .next()
        {
            let mut to_remove = String::with_capacity(1 + value.len());
            to_remove.push('_');
            to_remove.push_str(value);
            Self {
                code: error.error_code,
                name: error.error_message.replace(&to_remove, ""),
                value: Some(parsed_value),
                caused_by: None,
            }
        } else {
            Self {
                code: error.error_code,
                name: error.error_message.clone(),
                value: None,
                caused_by: None,
            }
        }
    }
}

impl RpcError {
    /// Matches on the name of the RPC error (case-sensitive).
    ///
    /// Useful in `match` arm guards. A single trailing or leading asterisk (`'*'`) is allowed,
    /// and will instead check if the error name starts (or ends with) the input parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// # let request_result = Result::<(), _>::Err(grammers_mtsender::RpcError {
    /// #     code: 400, name: "PHONE_CODE_INVALID".to_string(), value: None, caused_by: None });
    /// #
    /// match request_result {
    ///     Err(rpc_err) if rpc_err.is("SESSION_PASSWORD_NEEDED") => panic!(),
    ///     Err(rpc_err) if rpc_err.is("PHONE_CODE_*") => {},
    ///     _ => panic!()
    /// }
    /// ```
    pub fn is(&self, rpc_error: &str) -> bool {
        if let Some(rpc_error) = rpc_error.strip_suffix('*') {
            self.name.starts_with(rpc_error)
        } else if let Some(rpc_error) = rpc_error.strip_prefix('*') {
            self.name.ends_with(rpc_error)
        } else {
            self.name == rpc_error
        }
    }

    /// Attaches the [`tl::Identifiable::CONSTRUCTOR_ID`] of the
    /// request that caused this error to the error information.
    pub fn with_caused_by(mut self, constructor_id: u32) -> Self {
        self.caused_by = Some(constructor_id);
        self
    }
}

/// This error occurs when a Remote Procedure call was unsuccessful.
#[derive(Debug)]
pub enum InvocationError {
    /// The request invocation failed because it was invalid or the server
    /// could not process it successfully. If the server is suffering from
    /// temporary issues, the request may be retried after some time.
    Rpc(RpcError),

    /// Standard I/O error when reading the response.
    ///
    /// Telegram may kill the connection at any moment, but it is generally valid to retry
    /// the request at least once immediately, which will be done through a new connection.
    Io(io::Error),

    /// Error propagated from attempting to deserialize an invalid [`tl::Deserializable`].
    ///
    /// This occurs somewhat frequently when misusing a single session more than once at a time.
    /// Otherwise it might happen on bleeding-edge layers that have not had time to settle yet.
    Deserialize(mtp::DeserializeError),

    /// Error propagated from the underlying [`transport`].
    ///
    /// The most common variant is [`transport::Error::BadStatus`], which can occur when
    /// there's no valid Authorization Key (404) or too many connections have been made (429).
    Transport(transport::Error),

    /// The request was cancelled or dropped, and the results won't arrive.
    /// This may mean that the [`crate::SenderPoolRunner`] is no longer running.
    Dropped,

    /// The request was invoked in a datacenter that does not exist or is not known by the session.
    InvalidDc,

    /// The request caused the sender to connect to a new datacenter to be performed,
    /// but the Authorization Key generation process failed.
    Authentication(authentication::Error),
}

impl Clone for InvocationError {
    fn clone(&self) -> Self {
        match self {
            Self::Rpc(err) => Self::Rpc(err.clone()),
            Self::Io(err) => {
                // 安全地克隆 io::Error，避免访问可能无效的内部状态
                if let Some(os_err) = err.raw_os_error() {
                    Self::Io(io::Error::from_raw_os_error(os_err))
                } else {
                    Self::Io(io::Error::new(
                        err.kind(),
                        format!("IO error ({:?})", err.kind()),
                    ))
                }
            }
            Self::Deserialize(err) => Self::Deserialize(err.clone()),
            Self::Transport(err) => Self::Transport(err.clone()),
            Self::Dropped => Self::Dropped,
            Self::InvalidDc => Self::InvalidDc,
            Self::Authentication(err) => Self::Authentication(err.clone()),
        }
    }
}

impl std::error::Error for InvocationError {}

impl fmt::Display for InvocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rpc(err) => write!(f, "request error: {err}"),
            Self::Io(err) => {
                // 安全地格式化 io::Error，避免访问可能无效的内部状态
                if let Some(os_err) = err.raw_os_error() {
                    write!(f, "request error: OS error {}", os_err)
                } else {
                    write!(f, "request error: {:?}", err.kind())
                }
            }
            Self::Deserialize(err) => write!(f, "request error: {err}"),
            Self::Transport(err) => {
                // 安全地格式化 transport::Error，避免访问可能无效的内存
                match err {
                    transport::Error::MissingBytes => write!(f, "request error: transport-level: need more bytes"),
                    transport::Error::BadLen { got } => write!(f, "request error: transport-level: bad len (got {})", got),
                    transport::Error::BadSeq { expected, got } => {
                        write!(f, "request error: transport-level: bad seq (expected {}, got {})", expected, got)
                    }
                    transport::Error::BadCrc { expected, got } => {
                        write!(f, "request error: transport-level: bad crc (expected {}, got {})", expected, got)
                    }
                    transport::Error::BadStatus { status } => {
                        write!(f, "request error: transport-level: bad status (negative length -{})", status)
                    }
                }
            }
            Self::Dropped => write!(f, "request error: dropped (cancelled)"),
            Self::InvalidDc => write!(f, "request error: invalid dc"),
            Self::Authentication(err) => write!(f, "request error: {err}"),
        }
    }
}

impl From<ReadError> for InvocationError {
    fn from(error: ReadError) -> Self {
        match error {
            ReadError::Io(error) => {
                // 安全地转换 io::Error，避免访问可能无效的内存
                let os_err = error.raw_os_error();
                let kind = os_err.map(|_| io::ErrorKind::Other)
                    .or_else(|| {
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| error.kind()))
                            .ok()
                    })
                    .unwrap_or(io::ErrorKind::Other);

                let io_err = if let Some(os_err) = os_err {
                    io::Error::from_raw_os_error(os_err)
                } else {
                    io::Error::new(kind, format!("IO error ({:?})", kind))
                };
                Self::Io(io_err)
            }
            ReadError::Transport(error) => Self::Transport(error),
            ReadError::Deserialize(error) => Self::Deserialize(error),
        }
    }
}

impl From<mtp::DeserializeError> for InvocationError {
    fn from(error: mtp::DeserializeError) -> Self {
        Self::from(ReadError::from(error))
    }
}

impl From<transport::Error> for InvocationError {
    fn from(error: transport::Error) -> Self {
        Self::from(ReadError::from(error))
    }
}

impl From<tl::deserialize::Error> for InvocationError {
    fn from(error: tl::deserialize::Error) -> Self {
        Self::from(ReadError::from(error))
    }
}

impl From<io::Error> for InvocationError {
    fn from(error: io::Error) -> Self {
        // 安全地转换 io::Error，避免通过 ReadError 转换时访问无效内存
        // 直接创建 InvocationError::Io，避免中间转换
        let os_err = error.raw_os_error();
        let kind = os_err.map(|_| io::ErrorKind::Other)
            .or_else(|| {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| error.kind()))
                    .ok()
            })
            .unwrap_or(io::ErrorKind::Other);

        let io_err = if let Some(os_err) = os_err {
            io::Error::from_raw_os_error(os_err)
        } else {
            io::Error::new(kind, format!("IO error ({:?})", kind))
        };
        Self::Io(io_err)
    }
}

impl From<authentication::Error> for InvocationError {
    fn from(error: authentication::Error) -> Self {
        Self::Authentication(error)
    }
}

impl InvocationError {
    /// Matches on the name of the RPC error (case-sensitive).
    ///
    /// Useful in `match` arm guards. A single trailing or leading asterisk (`'*'`) is allowed,
    /// and will instead check if the error name starts (or ends with) the input parameter.
    ///
    /// If the error is not a RPC error, returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// # let request_result = Result::<(), _>::Err(grammers_mtsender::InvocationError::Rpc(
    /// #     grammers_mtsender::RpcError { code: 400, name: "PHONE_CODE_INVALID".to_string(), value: None, caused_by: None }));
    /// #
    /// match request_result {
    ///     Err(err) if err.is("SESSION_PASSWORD_NEEDED") => panic!(),
    ///     Err(err) if err.is("PHONE_CODE_*") => {},
    ///     _ => panic!()
    /// }
    /// ```
    #[inline]
    pub fn is(&self, rpc_error: &str) -> bool {
        match self {
            Self::Rpc(rpc) => rpc.is(rpc_error),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_rpc_error_parsing() {
        assert_eq!(
            RpcError::from(tl::types::RpcError {
                error_code: 400,
                error_message: "CHAT_INVALID".into(),
            }),
            RpcError {
                code: 400,
                name: "CHAT_INVALID".into(),
                value: None,
                caused_by: None,
            }
        );
    }
}
