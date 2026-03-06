import * as cloudflare from "@pulumi/cloudflare";
import { stackResourceName } from "../../shared/naming";
import { CloudflareTunnelOutputs, InfraConfig } from "../../shared/types";

export interface CloudflareDnsOutputs {
  dnsRecordId: cloudflare.DnsRecord["id"];
}

export function createCloudflareDnsRecord(
  config: InfraConfig,
  tunnel: CloudflareTunnelOutputs,
): CloudflareDnsOutputs {
  const record = new cloudflare.DnsRecord(stackResourceName(config, "origin-cname"), {
    zoneId: config.cloudflareZoneId,
    name: config.subdomain,
    type: "CNAME",
    content: tunnel.tunnelCnameTarget,
    proxied: true,
    ttl: 1,
  });

  return {
    dnsRecordId: record.id,
  };
}
