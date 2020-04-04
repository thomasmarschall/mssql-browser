use std::net::{ SocketAddr };
use std::error::Error;

/// An error that can be returned from the different browser operations
#[derive(Debug)]
pub enum BrowserError<SocketError: std::error::Error> {
    /// The underlying `tokio::net::UdpSocket` failed to bind.
    BindFailed(SocketError),

    /// Enabling the broadcast option on the `tokio::net::UdpSocket` failed.
    SetBroadcastFailed(SocketError),

    /// Sending the request datagram failed.
    SendFailed(SocketAddr, SocketError),

    /// Locking the `tokio::net::UdpSocket` to a specific endpoint via `tokio::net::UdpSocket::connect` failed.
    ConnectFailed(SocketAddr, SocketError),

    /// Receiving a datagram failed.
    ReceiveFailed(SocketError),

    /// The given instance name is too long.
    InstanceNameTooLong,

    /// The server send back an invalid response.
    ProtocolError(BrowserProtocolError)
}

impl<SocketError: Error> std::fmt::Display for BrowserError<SocketError> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use BrowserError::*;

        match self {
            BindFailed(err) => write!(f, "bind failed: {}", err),
            SetBroadcastFailed(err) => write!(f, "enabling broadcast option failed: {}", err),
            SendFailed(addr, err) => write!(f, "sending of datagram to '{}' failed: {}", addr, err),
            ConnectFailed(addr, err) => write!(f, "connect to '{}' failed: {}", addr, err),
            ReceiveFailed(err) => write!(f, "receiving of datagram failed: {}", err),
            InstanceNameTooLong => write!(f, "specified instance name is longer than {} bytes", super::MAX_INSTANCE_NAME_LEN),
            ProtocolError(e) => write!(f, "protocol error: {}", e)
        }
    }
}

impl<SocketError: Error> Error for BrowserError<SocketError> {
    fn cause(&self) -> Option<&dyn Error> {
        use BrowserError::*;

        match self {
            BindFailed(err) => Some(err),
            SetBroadcastFailed(err) => Some(err),
            SendFailed(_, err) => Some(err),
            ConnectFailed(_, err) => Some(err),
            ReceiveFailed(err) => Some(err),
            InstanceNameTooLong => None,
            ProtocolError(err) => Some(err)
        }
    }
}

/// Received an unexpected response from the server
#[derive(Debug)]
pub enum BrowserProtocolError {
    /// An unexpected token was received from the server
    UnexpectedToken {
        /// The token that was expected at this location
        expected: BrowserProtocolToken,

        /// The token that was found
        found: BrowserProtocolToken
    },

    /// The length of the datagram does not match the length
    /// specified in the packet header.
    LengthMismatch {
        /// The size, in bytes, of the datagram
        datagram: usize,

        /// The size, in bytes, specified in the packet header
        header: usize
    },

    /// Unexpected MBCS string encoding found in the received message
    InvalidUtf8(std::str::Utf8Error),

    /// There was extraneous data after the parsed message
    ExtraneousData(Vec<u8>)
}

impl std::fmt::Display for BrowserProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use BrowserProtocolError::*;

        match self {
            UnexpectedToken { expected, found } 
                => write!(f, "expected {}, but found {}", expected, found),
            LengthMismatch { datagram, header }
                => write!(f, "mismatch between datagram size {} bytes and size specified in header {} bytes", datagram, header),
            InvalidUtf8(err)
                => err.fmt(f),
            ExtraneousData(data)
                => write!(f, "{} unexpected trailing bytes", data.len())
        }
    }
}

impl Error for BrowserProtocolError { }

/// The value that was expected.
#[derive(Debug)]
pub enum BrowserProtocolToken {
    /// End of the datagram
    EndOfMessage,

    /// A literal string
    Literal(String),

    /// The message identifier specified in the header
    MessageIdentifier(u8),

    /// The message length specified in the header
    MessageLength,

    DacVersion(u8),
    DacPort,
    Identifier(BrowserProtocolField),
    ValueOf(BrowserProtocolField),
    TcpPort,
    ViaParameters,
    EndpointIdentifierOrSemicolon
}

impl std::fmt::Display for BrowserProtocolToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use BrowserProtocolToken::*;

        match self {
            EndOfMessage => write!(f, "end of message"),
            Literal(s) => write!(f, "'{}'", s),
            MessageIdentifier(v) => write!(f, "message identifier {:#X}", v),
            MessageLength => write!(f, "message length"),
            DacVersion(v) => write!(f, "dac version {}", v),
            DacPort => write!(f, "dac port"),
            Identifier(field) => write!(f, "identifier for field {:?}", field),
            ValueOf(field) => write!(f, "value for field {:?}", field),
            TcpPort => write!(f, "tcp port"),
            ViaParameters => write!(f, "via parameters"),
            EndpointIdentifierOrSemicolon => write!(f, "endpoint identifier or semicolon")
        }
    }  
}

/// Different fields found in a browser response
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum BrowserProtocolField {
    ServerName,
    InstanceName,
    IsClustered,
    Version,

    NamedPipeName,
    TcpPort,
    ViaMachineName,
    RpcComputerName,
    SpxServiceName,
    AppleTalkObjectName,
    BvItemName,
    BvGroupName,
    BvOrgName
}