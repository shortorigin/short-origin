import * as aws from "@pulumi/aws";
import * as pulumi from "@pulumi/pulumi";
import { createCloudflareDnsRecord } from "../cloudflare/dns";
import { AwsCloudflareWiringOutputs, AwsComputeOutputs, CloudflareTunnelOutputs, InfraConfig } from "../../shared/types";
import { composeFqdn, stackResourceName } from "../../shared/naming";

export interface AwsCloudflareWiringArgs {
  config: InfraConfig;
  compute: AwsComputeOutputs;
  tunnel: CloudflareTunnelOutputs;
}

export function createAwsCloudflareWiring(args: AwsCloudflareWiringArgs): AwsCloudflareWiringOutputs {
  const fqdn = composeFqdn(args.config);
  const dns = createCloudflareDnsRecord(args.config, args.tunnel);

  new aws.ssm.Parameter(stackResourceName(args.config, "fqdn-parameter"), {
    name: `${args.config.ssmPathPrefix}/${args.config.env}/public_fqdn`,
    type: "String",
    value: fqdn,
    overwrite: true,
  });

  new aws.ssm.Parameter(stackResourceName(args.config, "origin-private-ip-parameter"), {
    name: `${args.config.ssmPathPrefix}/${args.config.env}/origin_private_ip`,
    type: "String",
    value: args.compute.privateIp,
    overwrite: true,
  });

  return {
    fqdn: pulumi.output(fqdn),
    dnsRecordId: dns.dnsRecordId,
    tunnelId: args.tunnel.tunnelId,
  };
}
