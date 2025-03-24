use serde::{
    Deserialize,
    de::{self, Visitor},
};

pub const DEFAULT_PATH: &str = "~/.config/tunglo.toml";
#[derive(Deserialize, Debug, PartialEq)]
pub(crate) struct TungloConfig {
    pub storage: StorageConfig,
    pub tunnels: Vec<TunnelConfig>,
}
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct StorageConfig {
    #[serde(rename = "type")]
    pub storage_type: StorageType,
    pub rqlite: Option<RqliteStorageConfig>,
}
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct RqliteStorageConfig {
    pub host: EnvOrValue,
    pub user: Option<EnvOrValue>,
    pub password: Option<EnvOrValue>,
}
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct EnvOrValue {
    from_env: Option<String>,
    value: Option<String>,
}
impl EnvOrValue {
    pub fn get(&self) -> &str {
        // this should never panic (unwrap on None) because these values are checked
        // at deserialization time
        self.from_env.as_deref().or(self.value.as_deref()).unwrap()
    }
}
impl From<EnvOrValue> for String {
    fn from(value: EnvOrValue) -> Self {
        String::from(value.get())
    }
}
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub(crate) enum StorageType {
    #[serde(alias = "local", alias = "LOCAL")]
    Local,
    #[serde(alias = "rqlite", alias = "RQLITE")]
    Rqlite,
}
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct TunnelConfig {
    pub name: String,
    pub remote_ssh_address: String,
    pub remote_ssh_port: u16,
    pub remote_ssh_user: String,
    pub private_key_path: String,
    pub private_key_passphrase: Option<PrivateKeyPassphrase>,
    pub remote_interface_address: String,
    pub remote_interface_port: u16,
    pub to_address: String,
    pub to_port: u16,
    #[serde(rename = "type")]
    pub tun_type: TunnelType,
}
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub(crate) enum TunnelType {
    #[serde(alias = "http", alias = "HTTP")]
    Http,
    #[serde(alias = "http2", alias = "HTTP2")]
    Http2,
    #[serde(alias = "generic", alias = "GENERIC")]
    Generic,
}
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct PrivateKeyPassphrase {
    /// the private key is stored in platintext inside the tunnel configuration file
    /// (passhphrase-value)
    pub value: Option<String>,
    /// the private key must be fetched from an environmental variable (env-var-name)
    pub from_env: Option<String>,
}

impl<'de> Deserialize<'de> for EnvOrValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EnvOrValueVisitor;
        impl<'de> Visitor<'de> for EnvOrValueVisitor {
            type Value = EnvOrValue;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter
                    .write_str("a map with at least one between `from_env` or `value` set to Some")
            }
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut from_env = None;
                let mut value = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "from_env" => from_env = Some(map.next_value()?),
                        "value" => value = Some(map.next_value()?),
                        _ => return Err(de::Error::unknown_field(&key, &["from_env", "value"])),
                    }
                }

                if from_env.is_none() && value.is_none() {
                    return Err(de::Error::custom(
                        "at least one between `from_env` or `value` must be provided!",
                    ));
                }
                if from_env.is_some() && value.is_some() {
                    from_env = None; // value takes precedence
                }
                Ok(EnvOrValue { value, from_env })
            }
        }
        deserializer.deserialize_map(EnvOrValueVisitor)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn check_basic_deserialization() {
        let config_str = r#"
            [storage]
            type = "local"
            [[tunnels]]
            name = "another_web_service"
            remote_ssh_address = "1.1.1.1"
            remote_ssh_port = 123
            remote_ssh_user = "macca"
            private_key_path = "path"
            remote_interface_address = "1.0.0.0"
            remote_interface_port = 9002
            to_address = "localhost"
            to_port = 8082
            type = "http"
            [[tunnels]]
            name = "another_web_service"
            remote_ssh_address = "1.1.1.1"
            remote_ssh_port = 123
            remote_ssh_user = "macca"
            private_key_path = "path"
            remote_interface_address = "1.0.0.0"
            remote_interface_port = 9002
            to_address = "localhost"
            to_port = 8082
            type = "http2"
            [[tunnels]]
            name = "another_web_service"
            remote_ssh_address = "1.1.1.1"
            remote_ssh_port = 123
            remote_ssh_user = "macca"
            private_key_path = "path"
            remote_interface_address = "1.0.0.0"
            remote_interface_port = 9002
            to_address = "localhost"
            to_port = 8082
            type = "generic"
        "#;
        let rqlite_config = r#"
            [storage]
            type = "rqlite"
            [storage.rqlite]
            user.value = "macca"
            password.value = "pongle"
            host.value = "https://config-store:4001"
            [[tunnels]]
            name = "another_web_service"
            remote_ssh_address = "1.1.1.1"
            remote_ssh_port = 123
            remote_ssh_user = "macca"
            private_key_path = "path"
            remote_interface_address = "1.0.0.0"
            remote_interface_port = 9002
            to_address = "localhost"
            to_port = 8082
            type = "generic"
        "#;
        let parsed_config: Result<TungloConfig, toml::de::Error> = toml::from_str(config_str);
        let another_parsed_config: Result<TungloConfig, toml::de::Error> =
            toml::from_str(rqlite_config);
        assert!(&parsed_config.is_ok());
        assert!(&another_parsed_config.is_ok());
        let parsed_config = parsed_config.ok().unwrap();
        let another_parsed_config = another_parsed_config.ok().unwrap();
        assert_eq!(parsed_config.tunnels.len(), 3);
        let first_tunnel = parsed_config.tunnels.first().unwrap();
        let second_tunnel = parsed_config.tunnels.get(1).unwrap();
        let third_tunnel = parsed_config.tunnels.get(2).unwrap();
        assert_eq!(
            parsed_config.storage,
            StorageConfig {
                storage_type: StorageType::Local,
                rqlite: None,
            }
        );
        assert_eq!(
            another_parsed_config.storage,
            StorageConfig {
                storage_type: StorageType::Rqlite,
                rqlite: Some(RqliteStorageConfig {
                    password: Some(EnvOrValue {
                        from_env: None,
                        value: Some(String::from("pongle"))
                    }),
                    user: Some(EnvOrValue {
                        from_env: None,
                        value: Some(String::from("macca")),
                    }),
                    host: EnvOrValue {
                        from_env: None,
                        value: Some(String::from("https://config-store:4001")),
                    }
                })
            }
        );

        assert_eq!(
            *first_tunnel,
            TunnelConfig {
                name: String::from("another_web_service"),
                remote_ssh_address: String::from("1.1.1.1"),
                remote_ssh_port: 123,
                remote_ssh_user: String::from("macca"),
                private_key_path: String::from("path"),
                private_key_passphrase: None,
                remote_interface_address: String::from("1.0.0.0"),
                remote_interface_port: 9002,
                to_address: String::from("localhost"),
                to_port: 8082,
                tun_type: TunnelType::Http,
            }
        );
        assert_eq!(
            *second_tunnel,
            TunnelConfig {
                name: String::from("another_web_service"),
                remote_ssh_address: String::from("1.1.1.1"),
                remote_ssh_port: 123,
                remote_ssh_user: String::from("macca"),
                private_key_path: String::from("path"),
                private_key_passphrase: None,
                remote_interface_address: String::from("1.0.0.0"),
                remote_interface_port: 9002,
                to_address: String::from("localhost"),
                to_port: 8082,
                tun_type: TunnelType::Http2,
            }
        );
        assert_eq!(
            *third_tunnel,
            TunnelConfig {
                name: String::from("another_web_service"),
                remote_ssh_address: String::from("1.1.1.1"),
                remote_ssh_port: 123,
                remote_ssh_user: String::from("macca"),
                private_key_path: String::from("path"),
                private_key_passphrase: None,
                remote_interface_address: String::from("1.0.0.0"),
                remote_interface_port: 9002,
                to_address: String::from("localhost"),
                to_port: 8082,
                tun_type: TunnelType::Generic,
            }
        );
    }
    #[test]
    fn env_or_value_deserialization_ok() {
        let config_str_value = r#"
            [storage]
            type = "rqlite"
            [storage.rqlite]
            user.value = "macca"
            password.value = "password"
            host.value = "host"
            [[tunnels]]
            name = "another_web_service"
            remote_ssh_address = "1.1.1.1"
            remote_ssh_port = 123
            remote_ssh_user = "macca"
            private_key_path = "path"
            remote_interface_address = "1.0.0.0"
            remote_interface_port = 9002
            to_address = "localhost"
            to_port = 8082
            type = "http"

        "#;
        let config_str_env = r#"
            [storage]
            type = "rqlite"
            [storage.rqlite]
            user.from_env = "macca_env"
            password.from_env = "password_env"
            host.from_env = "host_env"
            [[tunnels]]
            name = "another_web_service"
            remote_ssh_address = "1.1.1.1"
            remote_ssh_port = 123
            remote_ssh_user = "macca"
            private_key_path = "path"
            remote_interface_address = "1.0.0.0"
            remote_interface_port = 9002
            to_address = "localhost"
            to_port = 8082
            type = "http"
        "#;
        let parsed_config_value: Result<TungloConfig, toml::de::Error> =
            toml::from_str(config_str_value);
        let parsed_config_env: Result<TungloConfig, toml::de::Error> =
            toml::from_str(config_str_env);
        assert!(parsed_config_value.is_ok());
        assert!(parsed_config_env.is_ok());
        assert!(
            parsed_config_value
                .as_ref()
                .ok()
                .unwrap()
                .storage
                .rqlite
                .is_some()
        );
        assert!(
            parsed_config_env
                .as_ref()
                .ok()
                .unwrap()
                .storage
                .rqlite
                .is_some()
        );
        let value_some = parsed_config_value.ok().unwrap().storage.rqlite.unwrap();
        let env_some = parsed_config_env.ok().unwrap().storage.rqlite.unwrap();
        assert_eq!(
            value_some,
            RqliteStorageConfig {
                host: EnvOrValue {
                    value: Some(String::from("host")),
                    from_env: None,
                },
                password: Some(EnvOrValue {
                    value: Some(String::from("password")),
                    from_env: None,
                }),
                user: Some(EnvOrValue {
                    value: Some(String::from("macca")),
                    from_env: None,
                })
            }
        );
        assert_eq!(
            env_some,
            RqliteStorageConfig {
                host: EnvOrValue {
                    from_env: Some(String::from("host_env")),
                    value: None,
                },
                password: Some(EnvOrValue {
                    from_env: Some(String::from("password_env")),
                    value: None,
                }),
                user: Some(EnvOrValue {
                    from_env: Some(String::from("macca_env")),
                    value: None,
                })
            }
        )
    }
    #[test]
    fn env_or_value_deserialization_value_preference_over_env() {
        let config_str_value = r#"
            [storage]
            type = "rqlite"
            [storage.rqlite]
            user.value = "yeah!"
            user.from_env = "shouldn't be selected"
            password.value = "password"
            host.value = "host"
            [[tunnels]]
            name = "another_web_service"
            remote_ssh_address = "1.1.1.1"
            remote_ssh_port = 123
            remote_ssh_user = "macca"
            private_key_path = "path"
            remote_interface_address = "1.0.0.0"
            remote_interface_port = 9002
            to_address = "localhost"
            to_port = 8082
            type = "http"

        "#;
        let parsed_config_value: Result<TungloConfig, toml::de::Error> =
            toml::from_str(config_str_value);
        assert!(parsed_config_value.is_ok());
        let config = parsed_config_value.ok().unwrap();
        let rqlite_config = config.storage.rqlite;
        assert!(rqlite_config.is_some());
        let rqlite_config = rqlite_config.unwrap();
        assert_eq!(
            rqlite_config,
            RqliteStorageConfig {
                host: EnvOrValue {
                    value: Some(String::from("host")),
                    from_env: None,
                },
                user: Some(EnvOrValue {
                    value: Some(String::from("yeah!")),
                    from_env: None,
                }),
                password: Some(EnvOrValue {
                    value: Some(String::from("password")),
                    from_env: None,
                })
            }
        );
    }
    #[test]
    fn check_passphrase_deserialization() {
        let config_str = r#"
            [storage]
            type = "local"
            [[tunnels]]
            name = "another_web_service"
            remote_ssh_address = "1.1.1.1"
            remote_ssh_port = 123
            remote_ssh_user = "macca"
            private_key_path = "path"
            remote_interface_address = "1.0.0.0"
            remote_interface_port = 9002
            to_address = "localhost"
            to_port = 8082
            type = "http"
            private_key_passphrase.value = "plaintext"
            [[tunnels]]
            name = "another_web_service"
            remote_ssh_address = "1.1.1.1"
            remote_ssh_port = 123
            remote_ssh_user = "macca"
            private_key_path = "path"
            remote_interface_address = "1.0.0.0"
            remote_interface_port = 9002
            to_address = "localhost"
            to_port = 8082
            type = "http"
            private_key_passphrase.from_env = "env_key"
        "#;
        let parsed_config: Result<TungloConfig, toml::de::Error> = toml::from_str(config_str);
        assert!(&parsed_config.is_ok());
        let parsed_config = parsed_config.ok().unwrap();
        assert_eq!(parsed_config.tunnels.len(), 2);
        let first_tunnel = parsed_config.tunnels.first().unwrap().clone();
        let second_tunnel = parsed_config.tunnels.get(1).unwrap().clone();
        assert!(first_tunnel.private_key_passphrase.is_some());
        assert!(second_tunnel.private_key_passphrase.is_some());
        let first_key = first_tunnel.private_key_passphrase.unwrap();
        let second_key = second_tunnel.private_key_passphrase.unwrap();
        assert_eq!(
            first_key,
            PrivateKeyPassphrase {
                value: Some(String::from("plaintext")),
                from_env: None,
            }
        );
        assert_eq!(
            second_key,
            PrivateKeyPassphrase {
                from_env: Some(String::from("env_key")),
                value: None,
            }
        );
    }
}
