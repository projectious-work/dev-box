use crate::config::AddonBundle;

/// Returns Dockerfile RUN commands for a given addon bundle.
pub fn dockerfile_commands(bundle: &AddonBundle) -> &'static str {
    match bundle {
        AddonBundle::Infrastructure => {
            r#"# Addon: infrastructure (OpenTofu, Ansible, Packer)
RUN curl -fsSL https://get.opentofu.org/install-opentofu.sh | sh -s -- --install-method standalone && \
    pip3 install --no-cache-dir ansible && \
    ARCH="$(dpkg --print-architecture)" && \
    curl -fsSL "https://releases.hashicorp.com/packer/1.11.2/packer_1.11.2_linux_${ARCH}.zip" -o /tmp/packer.zip && \
    unzip -q /tmp/packer.zip -d /usr/local/bin && rm /tmp/packer.zip"#
        }
        AddonBundle::Kubernetes => {
            r#"# Addon: kubernetes (kubectl, Helm, k9s, Kustomize)
RUN ARCH="$(dpkg --print-architecture)" && \
    curl -fsSL "https://dl.k8s.io/release/$(curl -fsSL https://dl.k8s.io/release/stable.txt)/bin/linux/${ARCH}/kubectl" -o /usr/local/bin/kubectl && \
    chmod +x /usr/local/bin/kubectl && \
    curl -fsSL https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash && \
    curl -fsSL "https://github.com/derailed/k9s/releases/latest/download/k9s_Linux_${ARCH}.tar.gz" | tar xz -C /usr/local/bin k9s && \
    curl -fsSL "https://raw.githubusercontent.com/kubernetes-sigs/kustomize/master/hack/install_kustomize.sh" | bash && \
    mv kustomize /usr/local/bin/"#
        }
        AddonBundle::CloudAws => {
            r#"# Addon: cloud-aws (AWS CLI v2)
RUN ARCH="$(uname -m)" && \
    curl -fsSL "https://awscli.amazonaws.com/awscli-exe-linux-${ARCH}.zip" -o /tmp/awscli.zip && \
    unzip -q /tmp/awscli.zip -d /tmp && \
    /tmp/aws/install && \
    rm -rf /tmp/aws /tmp/awscli.zip"#
        }
        AddonBundle::CloudGcp => {
            r#"# Addon: cloud-gcp (Google Cloud CLI)
RUN curl -fsSL https://packages.cloud.google.com/apt/doc/apt-key.gpg | gpg --dearmor -o /usr/share/keyrings/cloud.google.gpg && \
    echo "deb [signed-by=/usr/share/keyrings/cloud.google.gpg] https://packages.cloud.google.com/apt cloud-sdk main" > /etc/apt/sources.list.d/google-cloud-sdk.list && \
    apt-get update && apt-get install -y --no-install-recommends google-cloud-cli && \
    rm -rf /var/lib/apt/lists/*"#
        }
        AddonBundle::CloudAzure => {
            r#"# Addon: cloud-azure (Azure CLI)
RUN pip3 install --no-cache-dir azure-cli"#
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infrastructure_commands_contain_expected_tools() {
        let cmds = dockerfile_commands(&AddonBundle::Infrastructure);
        assert!(cmds.contains("opentofu"), "should install OpenTofu");
        assert!(cmds.contains("ansible"), "should install Ansible");
        assert!(cmds.contains("packer"), "should install Packer");
    }

    #[test]
    fn kubernetes_commands_contain_expected_tools() {
        let cmds = dockerfile_commands(&AddonBundle::Kubernetes);
        assert!(cmds.contains("kubectl"), "should install kubectl");
        assert!(cmds.contains("helm"), "should install Helm");
        assert!(cmds.contains("k9s"), "should install k9s");
        assert!(cmds.contains("kustomize"), "should install Kustomize");
    }

    #[test]
    fn cloud_aws_commands_contain_expected_tools() {
        let cmds = dockerfile_commands(&AddonBundle::CloudAws);
        assert!(cmds.contains("awscli"), "should install AWS CLI");
    }

    #[test]
    fn cloud_gcp_commands_contain_expected_tools() {
        let cmds = dockerfile_commands(&AddonBundle::CloudGcp);
        assert!(cmds.contains("google-cloud"), "should install Google Cloud CLI");
    }

    #[test]
    fn cloud_azure_commands_contain_expected_tools() {
        let cmds = dockerfile_commands(&AddonBundle::CloudAzure);
        assert!(cmds.contains("azure-cli"), "should install Azure CLI");
    }

    #[test]
    fn all_commands_start_with_comment() {
        let bundles = [
            AddonBundle::Infrastructure,
            AddonBundle::Kubernetes,
            AddonBundle::CloudAws,
            AddonBundle::CloudGcp,
            AddonBundle::CloudAzure,
        ];
        for bundle in &bundles {
            let cmds = dockerfile_commands(bundle);
            assert!(
                cmds.starts_with("# Addon:"),
                "{} commands should start with '# Addon:' comment",
                bundle
            );
        }
    }
}
