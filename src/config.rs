use serde::Deserialize;

pub const DEFAULT_PATH: &str = "~/.config/tunglo.toml";
#[derive(Deserialize, Debug)]
pub(crate) struct TungloConfig {
    pub tunnels: Vec<TunnelConfig>,
}
#[derive(Deserialize, Debug, Clone)]
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

#[cfg(test)]
mod tests {
    use crate::tunneling::tunnel::TunnelError;

    use super::*;
    #[test]
    fn check_basic_deserialization() {
        let config_str = r#"
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

        "#;
        let parsed_config: Result<TungloConfig, toml::de::Error> = toml::from_str(&config_str);
        assert!(&parsed_config.is_ok());
        let parsed_config = parsed_config.ok().unwrap();
        assert_eq!(parsed_config.tunnels.len(), 1);
        let tunnel = parsed_config.tunnels.first().unwrap();
        assert_eq!(tunnel.name, "another_web_service".to_string());
        assert_eq!(tunnel.remote_ssh_address, "1.1.1.1".to_string());
        assert_eq!(tunnel.remote_ssh_port, 123);
        assert_eq!(tunnel.remote_ssh_user, "macca".to_string());
        assert_eq!(tunnel.private_key_path, "path".to_string());
        assert_eq!(tunnel.remote_interface_port, 9002);
        assert_eq!(tunnel.to_address, "localhost".to_string());
        assert_eq!(tunnel.to_port, 8082);
        assert_eq!(tunnel.tun_type, TunnelType::Http);
    }
    #[test]
    fn check_passphrase_deserialization() {
        let config_str = r#"
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
            [tunnels.private_key_passphrase]
            value = "plaintext"
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
            [tunnels.private_key_passphrase]
            from_env = "env_key"
        "#;
        let parsed_config: Result<TungloConfig, toml::de::Error> = toml::from_str(&config_str);
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
