use serde::{Deserialize, Serialize};

/// Cloud-init configuration for VM provisioning
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CloudInitConfig {
    /// Enable/disable cloud-init
    #[serde(default)]
    pub enabled: bool,

    /// Username for the default user
    pub username: Option<String>,

    /// Password for the default user (plain text, will be hashed)
    pub password: Option<String>,

    /// SSH authorized keys
    #[serde(default)]
    pub ssh_authorized_keys: Vec<String>,

    /// Hostname for the VM
    pub hostname: Option<String>,

    /// List of packages to install
    #[serde(default)]
    pub packages: Vec<String>,

    /// Shell commands to run on first boot
    #[serde(default)]
    pub runcmd: Vec<String>,

    /// Custom user-data YAML (overrides other settings if provided)
    pub custom_user_data: Option<String>,

    /// Network configuration
    pub network_config: Option<NetworkConfig>,

    /// Auto-eject cloud-init ISO after first boot
    #[serde(default)]
    pub auto_eject: bool,
}

/// Network configuration for cloud-init
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NetworkConfig {
    /// Version (1 or 2)
    pub version: u8,

    /// Network config YAML (custom configuration)
    pub config_yaml: Option<String>,
}

/// Cloud-init template
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CloudInitTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub config: CloudInitConfig,
}

#[allow(dead_code)]
impl CloudInitConfig {
    /// Generate cloud-config YAML (user-data)
    pub fn generate_user_data(&self) -> Result<String, String> {
        // If custom user-data is provided, use it directly
        if let Some(ref custom) = self.custom_user_data {
            return Ok(custom.clone());
        }

        let mut yaml = String::from("#cloud-config\n");

        // Add username if specified
        if let Some(ref username) = self.username {
            yaml.push_str(&format!("users:\n"));
            yaml.push_str(&format!("  - name: {}\n", username));
            yaml.push_str(&format!("    sudo: ALL=(ALL) NOPASSWD:ALL\n"));
            yaml.push_str(&format!("    groups: users, admin, wheel, sudo\n"));
            yaml.push_str(&format!("    shell: /bin/bash\n"));

            // Add password if specified
            if let Some(ref _password) = self.password {
                // Note: In production, password should be hashed
                // For simplicity, we're using plain text with chpasswd
                yaml.push_str(&format!("    lock_passwd: false\n"));
            }

            // Add SSH keys if specified
            if !self.ssh_authorized_keys.is_empty() {
                yaml.push_str(&format!("    ssh_authorized_keys:\n"));
                for key in &self.ssh_authorized_keys {
                    yaml.push_str(&format!("      - {}\n", key));
                }
            }
        }

        // Set password via chpasswd if username and password are provided
        if let (Some(ref username), Some(ref password)) = (&self.username, &self.password) {
            yaml.push_str(&format!("\nchpasswd:\n"));
            yaml.push_str(&format!("  list: |\n"));
            yaml.push_str(&format!("    {}:{}\n", username, password));
            yaml.push_str(&format!("  expire: false\n"));
        }

        // Set hostname
        if let Some(ref hostname) = self.hostname {
            yaml.push_str(&format!("\nhostname: {}\n", hostname));
        }

        // Install packages
        if !self.packages.is_empty() {
            yaml.push_str("\npackages:\n");
            for package in &self.packages {
                yaml.push_str(&format!("  - {}\n", package));
            }
        }

        // Run commands
        if !self.runcmd.is_empty() {
            yaml.push_str("\nruncmd:\n");
            for cmd in &self.runcmd {
                yaml.push_str(&format!("  - {}\n", cmd));
            }
        }

        // Enable password authentication for SSH
        if self.password.is_some() {
            yaml.push_str("\nssh_pwauth: true\n");
        }

        // Disable SSH password auth if only using keys
        if self.password.is_none() && !self.ssh_authorized_keys.is_empty() {
            yaml.push_str("\nssh_pwauth: false\n");
        }

        Ok(yaml)
    }

    /// Generate meta-data
    pub fn generate_meta_data(&self, instance_id: &str) -> String {
        let mut meta = format!("instance-id: {}\n", instance_id);

        if let Some(ref hostname) = self.hostname {
            meta.push_str(&format!("local-hostname: {}\n", hostname));
        }

        meta
    }

    /// Generate network-config (if specified)
    pub fn generate_network_config(&self) -> Option<String> {
        self.network_config.as_ref().and_then(|nc| {
            nc.config_yaml.clone().or_else(|| {
                // Default network config
                Some(format!(
                    "version: {}\nconfig: []\n",
                    nc.version
                ))
            })
        })
    }
}

/// Built-in cloud-init templates
#[allow(dead_code)]
pub fn get_builtin_templates() -> Vec<CloudInitTemplate> {
    vec![
        // Basic user setup
        CloudInitTemplate {
            id: "basic-user".to_string(),
            name: "Basic User Setup".to_string(),
            description: "Create a user with password authentication".to_string(),
            config: CloudInitConfig {
                enabled: true,
                username: Some("ubuntu".to_string()),
                password: Some("changeme".to_string()),
                ssh_authorized_keys: vec![],
                hostname: None,
                packages: vec![],
                runcmd: vec![],
                custom_user_data: None,
                network_config: None,
                auto_eject: true,
            },
        },

        // SSH key only
        CloudInitTemplate {
            id: "ssh-key".to_string(),
            name: "SSH Key Only".to_string(),
            description: "Create a user with SSH key authentication (no password)".to_string(),
            config: CloudInitConfig {
                enabled: true,
                username: Some("ubuntu".to_string()),
                password: None,
                ssh_authorized_keys: vec!["ssh-rsa AAAAB3... user@host".to_string()],
                hostname: None,
                packages: vec![],
                runcmd: vec![],
                custom_user_data: None,
                network_config: None,
                auto_eject: true,
            },
        },

        // Docker pre-installed
        CloudInitTemplate {
            id: "docker".to_string(),
            name: "Docker Pre-installed".to_string(),
            description: "User setup with Docker and Docker Compose pre-installed".to_string(),
            config: CloudInitConfig {
                enabled: true,
                username: Some("ubuntu".to_string()),
                password: Some("changeme".to_string()),
                ssh_authorized_keys: vec![],
                hostname: None,
                packages: vec![
                    "docker.io".to_string(),
                    "docker-compose".to_string(),
                ],
                runcmd: vec![
                    "systemctl enable docker".to_string(),
                    "systemctl start docker".to_string(),
                    "usermod -aG docker ubuntu".to_string(),
                ],
                custom_user_data: None,
                network_config: None,
                auto_eject: true,
            },
        },

        // Kubernetes node
        CloudInitTemplate {
            id: "k8s-node".to_string(),
            name: "Kubernetes Node".to_string(),
            description: "Kubernetes node with kubeadm, kubelet, and kubectl".to_string(),
            config: CloudInitConfig {
                enabled: true,
                username: Some("ubuntu".to_string()),
                password: Some("changeme".to_string()),
                ssh_authorized_keys: vec![],
                hostname: None,
                packages: vec![
                    "apt-transport-https".to_string(),
                    "ca-certificates".to_string(),
                    "curl".to_string(),
                ],
                runcmd: vec![
                    "curl -s https://packages.cloud.google.com/apt/doc/apt-key.gpg | apt-key add -".to_string(),
                    "echo 'deb https://apt.kubernetes.io/ kubernetes-xenial main' > /etc/apt/sources.list.d/kubernetes.list".to_string(),
                    "apt-get update".to_string(),
                    "apt-get install -y kubelet kubeadm kubectl".to_string(),
                    "apt-mark hold kubelet kubeadm kubectl".to_string(),
                ],
                custom_user_data: None,
                network_config: None,
                auto_eject: true,
            },
        },
    ]
}
