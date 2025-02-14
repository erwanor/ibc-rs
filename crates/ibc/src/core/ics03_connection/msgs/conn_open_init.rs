use crate::prelude::*;

use core::time::Duration;

use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenInit as RawMsgConnectionOpenInit;
use ibc_proto::protobuf::Protobuf;

use crate::core::ics03_connection::connection::Counterparty;
use crate::core::ics03_connection::error::ConnectionError;
use crate::core::ics03_connection::version::Version;
use crate::core::ics24_host::identifier::ClientId;
use crate::signer::Signer;
use crate::tx_msg::Msg;

pub const TYPE_URL: &str = "/ibc.core.connection.v1.MsgConnectionOpenInit";

/// Per our convention, this message is sent to chain A.
/// The handler will check proofs of chain B.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgConnectionOpenInit {
    /// ClientId on chain A that the connection is being opened for
    pub client_id_on_a: ClientId,
    pub counterparty: Counterparty,
    pub version: Option<Version>,
    pub delay_period: Duration,
    pub signer: Signer,
}

impl Msg for MsgConnectionOpenInit {
    type Raw = RawMsgConnectionOpenInit;

    fn type_url(&self) -> String {
        TYPE_URL.to_string()
    }
}

impl Protobuf<RawMsgConnectionOpenInit> for MsgConnectionOpenInit {}

impl TryFrom<RawMsgConnectionOpenInit> for MsgConnectionOpenInit {
    type Error = ConnectionError;

    fn try_from(msg: RawMsgConnectionOpenInit) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id_on_a: msg
                .client_id
                .parse()
                .map_err(ConnectionError::InvalidIdentifier)?,
            counterparty: msg
                .counterparty
                .ok_or(ConnectionError::MissingCounterparty)?
                .try_into()?,
            version: msg.version.map(|version| version.try_into()).transpose()?,
            delay_period: Duration::from_nanos(msg.delay_period),
            signer: msg.signer.parse().map_err(ConnectionError::Signer)?,
        })
    }
}

impl From<MsgConnectionOpenInit> for RawMsgConnectionOpenInit {
    fn from(ics_msg: MsgConnectionOpenInit) -> Self {
        RawMsgConnectionOpenInit {
            client_id: ics_msg.client_id_on_a.as_str().to_string(),
            counterparty: Some(ics_msg.counterparty.into()),
            version: ics_msg.version.map(|version| version.into()),
            delay_period: ics_msg.delay_period.as_nanos() as u64,
            signer: ics_msg.signer.to_string(),
        }
    }
}

#[cfg(test)]
pub mod test_util {
    use crate::core::ics03_connection::connection::Counterparty;
    use crate::prelude::*;
    use ibc_proto::ibc::core::connection::v1::{
        MsgConnectionOpenInit as RawMsgConnectionOpenInit, Version as RawVersion,
    };

    use crate::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
    use crate::core::ics03_connection::msgs::test_util::get_dummy_raw_counterparty;
    use crate::core::ics03_connection::version::Version;
    use crate::core::ics24_host::identifier::ClientId;
    use crate::test_utils::get_dummy_bech32_account;

    /// Extends the implementation with additional helper methods.
    impl MsgConnectionOpenInit {
        /// Returns a new `MsgConnectionOpenInit` with dummy values.
        pub fn new_dummy() -> Self {
            MsgConnectionOpenInit::try_from(get_dummy_raw_msg_conn_open_init()).unwrap()
        }

        /// Setter for `client_id`. Amenable to chaining, since it consumes the input message.
        pub fn with_client_id(self, client_id: ClientId) -> Self {
            MsgConnectionOpenInit {
                client_id_on_a: client_id,
                ..self
            }
        }

        /// Setter for `counterparty`. Amenable to chaining, since it consumes the input message.\
        pub fn with_counterparty_conn_id(self, counterparty_conn_id: u64) -> Self {
            let counterparty =
                Counterparty::try_from(get_dummy_raw_counterparty(Some(counterparty_conn_id)))
                    .unwrap();
            MsgConnectionOpenInit {
                counterparty,
                ..self
            }
        }

        pub fn with_version(self, identifier: Option<&str>) -> Self {
            let version = match identifier {
                Some(v) => Version::try_from(RawVersion {
                    identifier: v.to_string(),
                    features: vec![],
                })
                .unwrap()
                .into(),
                None => None,
            };
            MsgConnectionOpenInit { version, ..self }
        }
    }

    /// Returns a dummy message, for testing only.
    /// Other unit tests may import this if they depend on a MsgConnectionOpenInit.
    pub fn get_dummy_raw_msg_conn_open_init() -> RawMsgConnectionOpenInit {
        RawMsgConnectionOpenInit {
            client_id: ClientId::default().to_string(),
            counterparty: Some(get_dummy_raw_counterparty(None)),
            version: Some(Version::default().into()),
            delay_period: 0,
            signer: get_dummy_bech32_account(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use test_log::test;

    use ibc_proto::ibc::core::connection::v1::Counterparty as RawCounterparty;
    use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenInit as RawMsgConnectionOpenInit;

    use super::MsgConnectionOpenInit;
    use crate::core::ics03_connection::msgs::conn_open_init::test_util::get_dummy_raw_msg_conn_open_init;
    use crate::core::ics03_connection::msgs::test_util::get_dummy_raw_counterparty;

    #[test]
    fn parse_connection_open_init_msg() {
        #[derive(Clone, Debug, PartialEq)]
        struct Test {
            name: String,
            raw: RawMsgConnectionOpenInit,
            want_pass: bool,
        }

        let default_init_msg = get_dummy_raw_msg_conn_open_init();

        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_init_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Bad client id, name too short".to_string(),
                raw: RawMsgConnectionOpenInit {
                    client_id: "client".to_string(),
                    ..default_init_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad destination connection id, name too long".to_string(),
                raw: RawMsgConnectionOpenInit {
                    counterparty: Some(RawCounterparty {
                        connection_id:
                            "abcdefghijksdffjssdkflweldflsfladfsfwjkrekcmmsdfsdfjflddmnopqrstu"
                                .to_string(),
                        ..get_dummy_raw_counterparty(None)
                    }),
                    ..default_init_msg
                },
                want_pass: false,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let msg = MsgConnectionOpenInit::try_from(test.raw.clone());

            assert_eq!(
                test.want_pass,
                msg.is_ok(),
                "MsgConnOpenInit::new failed for test {}, \nmsg {:?} with error {:?}",
                test.name,
                test.raw,
                msg.err(),
            );
        }
    }

    #[test]
    fn to_and_from() {
        let raw = get_dummy_raw_msg_conn_open_init();
        let msg = MsgConnectionOpenInit::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgConnectionOpenInit::from(msg.clone());
        let msg_back = MsgConnectionOpenInit::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);

        // Check if handler sets counterparty connection id to `None`
        // in case relayer passes `MsgConnectionOpenInit` message with it set to `Some(_)`.
        let raw_with_counterpary_conn_id_some = get_dummy_raw_msg_conn_open_init();
        let msg_with_counterpary_conn_id_some =
            MsgConnectionOpenInit::try_from(raw_with_counterpary_conn_id_some).unwrap();
        let raw_with_counterpary_conn_id_some_back =
            RawMsgConnectionOpenInit::from(msg_with_counterpary_conn_id_some.clone());
        let msg_with_counterpary_conn_id_some_back =
            MsgConnectionOpenInit::try_from(raw_with_counterpary_conn_id_some_back.clone())
                .unwrap();
        assert_eq!(
            raw_with_counterpary_conn_id_some_back
                .counterparty
                .unwrap()
                .connection_id,
            "".to_string()
        );
        assert_eq!(
            msg_with_counterpary_conn_id_some,
            msg_with_counterpary_conn_id_some_back
        );
    }
}
