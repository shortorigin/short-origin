import * as pulumi from "@pulumi/pulumi";

export interface InfraConfig {
  projectName: string;
  env: "dev" | "stage" | "prod";
  awsRegion: string;
  vpcCidr: string;
  publicSubnetCidr: string;
  privateSubnetCidr: string;
  instanceType: string;
  instanceAmiArch: "arm64";
  rootDomain: string;
  subdomain: string;
  cloudflareAccountId: string;
  cloudflareZoneId: string;
  tunnelName: string;
  servicePort: number;
  ssmPathPrefix: string;
  wasmcloudVersion: string;
  surrealdbVersion: string;
}

export interface InfraSecrets {
  tunnelSecret: pulumi.Output<string>;
  surrealdbRootPassword: pulumi.Output<string>;
}

export interface AwsNetworkOutputs {
  vpcId: pulumi.Output<string>;
  publicSubnetId: pulumi.Output<string>;
  privateSubnetId: pulumi.Output<string>;
  instanceSecurityGroupId: pulumi.Output<string>;
}

export interface AwsIamOutputs {
  instanceProfileName: pulumi.Output<string>;
  instanceRoleArn: pulumi.Output<string>;
}

export interface AwsComputeOutputs {
  instanceId: pulumi.Output<string>;
  privateIp: pulumi.Output<string>;
  instanceRoleArn: pulumi.Output<string>;
}

export interface CloudflareTunnelOutputs {
  tunnelId: pulumi.Output<string>;
  tunnelCnameTarget: pulumi.Output<string>;
}

export interface AwsCloudflareWiringOutputs {
  fqdn: pulumi.Output<string>;
  dnsRecordId: pulumi.Output<string>;
  tunnelId: pulumi.Output<string>;
}
