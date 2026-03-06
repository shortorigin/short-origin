import * as cloudflare from "@pulumi/cloudflare";
import * as pulumi from "@pulumi/pulumi";
import { stackResourceName } from "../../shared/naming";
import { CloudflareTunnelOutputs, InfraConfig, InfraSecrets } from "../../shared/types";

export interface CreateCloudflareTunnelArgs {
  config: InfraConfig;
  secrets: InfraSecrets;
  fqdn: string;
}

export function createCloudflareTunnel(args: CreateCloudflareTunnelArgs): CloudflareTunnelOutputs {
  const tunnel = new cloudflare.ZeroTrustTunnelCloudflared(stackResourceName(args.config, "tunnel"), {
    accountId: args.config.cloudflareAccountId,
    name: args.config.tunnelName,
    configSrc: "local",
    tunnelSecret: args.secrets.tunnelSecret,
  });

  new cloudflare.ZeroTrustTunnelCloudflaredConfig(stackResourceName(args.config, "tunnel-config"), {
    accountId: args.config.cloudflareAccountId,
    tunnelId: tunnel.id,
    source: "local",
    config: {
      ingresses: [
        {
          hostname: args.fqdn,
          service: pulumi.interpolate`http://127.0.0.1:${args.config.servicePort}`,
        },
        {
          service: "http_status:404",
        },
      ],
    },
  });

  return {
    tunnelId: tunnel.id,
    tunnelCnameTarget: pulumi.interpolate`${tunnel.id}.cfargotunnel.com`,
  };
}
