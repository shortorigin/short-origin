import * as pulumi from "@pulumi/pulumi";
import { loadConfig } from "./config/load";
import { composeFqdn } from "./shared/naming";
import { defaultTags } from "./shared/tags";
import { createAwsNetwork } from "./modules/aws/network";
import { createEc2Iam } from "./modules/aws/iam";
import { createCompute } from "./modules/aws/compute";
import { createObservability } from "./modules/aws/observability";
import { createCloudflareTunnel } from "./modules/cloudflare/tunnel";
import { createEdgeSecurityBaseline } from "./modules/cloudflare/edgeSecurity";
import { createAwsCloudflareWiring } from "./modules/wiring/awsCloudflare";

const loaded = loadConfig();
const config = loaded.config;
const secrets = loaded.secrets;

const tags = defaultTags(config);
const serviceFqdn = composeFqdn(config);

const network = createAwsNetwork(config, tags);
const iam = createEc2Iam(config, tags);

const tunnel = createCloudflareTunnel({
  config,
  secrets,
  fqdn: serviceFqdn,
});

const compute = createCompute({
  config,
  secrets,
  tags,
  network,
  iam,
  fqdn: serviceFqdn,
  tunnelId: tunnel.tunnelId,
});

createObservability({
  config,
  tags,
  instanceId: compute.instanceId,
});

createEdgeSecurityBaseline(config);

const wiring = createAwsCloudflareWiring({
  config,
  compute,
  tunnel,
});

export const environment = config.env;
export const region = config.awsRegion;
export const fqdn = wiring.fqdn;
export const awsInstanceId = compute.instanceId;
export const awsPrivateIp = compute.privateIp;
export const cloudflareTunnelId = wiring.tunnelId;
export const pulumiStateBackendHint = pulumi.output("s3://<state-bucket>?region=us-west-2&awssdk=v2");
