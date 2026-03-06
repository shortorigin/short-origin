import * as pulumi from "@pulumi/pulumi";

export interface CloudInitArgs {
  fqdn: string;
  servicePort: number;
  cloudflareAccountId: string;
  tunnelId: pulumi.Output<string>;
  tunnelSecret: pulumi.Output<string>;
  surrealdbRootPassword: pulumi.Output<string>;
  wasmcloudVersion: string;
  surrealdbVersion: string;
}

export function renderCloudInit(args: CloudInitArgs): pulumi.Output<string> {
  return pulumi.all([
    args.tunnelId,
    args.tunnelSecret,
    args.surrealdbRootPassword,
  ]).apply(([tunnelId, tunnelSecret, surrealdbRootPassword]) => {
    const hostLabel = args.fqdn.replace(/\./g, "-");
    return `#!/bin/bash
set -euxo pipefail

hostnamectl set-hostname short-origin-${hostLabel}

# Base packages
dnf update -y
dnf install -y curl unzip tar jq amazon-cloudwatch-agent

# Install cloudflared
rpm --import https://pkg.cloudflare.com/cloudflare-main.gpg
cat >/etc/yum.repos.d/cloudflared.repo <<'REPO'
[cloudflared]
name=cloudflared
baseurl=https://pkg.cloudflare.com/cloudflared
enabled=1
gpgcheck=1
gpgkey=https://pkg.cloudflare.com/cloudflare-main.gpg
REPO
dnf install -y cloudflared

# Install wasmCloud (arm64)
WASMCLOUD_VERSION="${args.wasmcloudVersion}"
curl -L "https://github.com/wasmCloud/wasmCloud/releases/download/v${args.wasmcloudVersion}/wasmcloud-${args.wasmcloudVersion}-aarch64-unknown-linux-musl.tar.gz" -o /tmp/wasmcloud.tgz
mkdir -p /opt/wasmcloud
tar -xzf /tmp/wasmcloud.tgz -C /opt/wasmcloud --strip-components=1
ln -sf /opt/wasmcloud/wasmcloud_host /usr/local/bin/wasmcloud_host

# Install SurrealDB (arm64)
SURREALDB_VERSION="${args.surrealdbVersion}"
curl -L "https://github.com/surrealdb/surrealdb/releases/download/v${args.surrealdbVersion}/surreal-v${args.surrealdbVersion}.linux-arm64.tgz" -o /tmp/surrealdb.tgz
mkdir -p /opt/surrealdb
tar -xzf /tmp/surrealdb.tgz -C /opt/surrealdb --strip-components=1
ln -sf /opt/surrealdb/surreal /usr/local/bin/surreal

mkdir -p /var/lib/surrealdb
mkdir -p /etc/cloudflared

cat >/etc/systemd/system/surrealdb.service <<'UNIT'
[Unit]
Description=SurrealDB
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
Environment=SURREAL_USER=root
Environment=SURREAL_PASS=${surrealdbRootPassword}
ExecStart=/usr/local/bin/surreal start --log info --bind 127.0.0.1:8000 --user root --pass ${surrealdbRootPassword} surrealkv:///var/lib/surrealdb/surreal.db
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
UNIT

cat >/etc/systemd/system/wasmcloud.service <<'UNIT'
[Unit]
Description=wasmCloud Host
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/wasmcloud_host
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
UNIT

cat >/etc/cloudflared/${tunnelId}.json <<'JSON'
{
  "AccountTag": "${args.cloudflareAccountId}",
  "TunnelID": "${tunnelId}",
  "TunnelSecret": "${tunnelSecret}"
}
JSON

cat >/etc/cloudflared/config.yml <<'CFG'
tunnel: ${tunnelId}
credentials-file: /etc/cloudflared/${tunnelId}.json
ingress:
  - hostname: ${args.fqdn}
    service: http://127.0.0.1:${args.servicePort}
  - service: http_status:404
CFG

cat >/etc/systemd/system/cloudflared.service <<'UNIT'
[Unit]
Description=Cloudflare Tunnel
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/bin/cloudflared tunnel --config /etc/cloudflared/config.yml run
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
UNIT

systemctl daemon-reload
systemctl enable surrealdb wasmcloud cloudflared
systemctl restart surrealdb wasmcloud cloudflared
`;
  });
}
