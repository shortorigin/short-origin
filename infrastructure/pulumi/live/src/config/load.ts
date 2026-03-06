import * as pulumi from "@pulumi/pulumi";
import { validateInfraConfig } from "./schema";
import { InfraConfig, InfraSecrets } from "../shared/types";

export interface LoadedInfra {
  config: InfraConfig;
  secrets: InfraSecrets;
}

export function loadConfig(): LoadedInfra {
  const awsConfig = new pulumi.Config("aws");
  const config = new pulumi.Config("short-origin");

  const parsed = validateInfraConfig({
    projectName: pulumi.getProject(),
    env: config.require("env") as "dev" | "stage" | "prod",
    awsRegion: awsConfig.require("region"),
    vpcCidr: config.require("vpcCidr"),
    publicSubnetCidr: config.require("publicSubnetCidr"),
    privateSubnetCidr: config.require("privateSubnetCidr"),
    instanceType: config.require("instanceType"),
    instanceAmiArch: "arm64",
    rootDomain: config.require("rootDomain"),
    subdomain: config.require("subdomain"),
    cloudflareAccountId: config.require("cloudflareAccountId"),
    cloudflareZoneId: config.require("cloudflareZoneId"),
    tunnelName: config.require("tunnelName"),
    servicePort: Number(config.require("servicePort")),
    ssmPathPrefix: config.require("ssmPathPrefix"),
    wasmcloudVersion: config.require("wasmcloudVersion"),
    surrealdbVersion: config.require("surrealdbVersion"),
  });

  return {
    config: parsed,
    secrets: {
      tunnelSecret: config.requireSecret("tunnelSecret"),
      surrealdbRootPassword: config.requireSecret("surrealdbRootPassword"),
    },
  };
}
