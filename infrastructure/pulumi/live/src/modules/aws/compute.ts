import * as aws from "@pulumi/aws";
import * as pulumi from "@pulumi/pulumi";
import { stackResourceName } from "../../shared/naming";
import { AwsComputeOutputs, AwsIamOutputs, AwsNetworkOutputs, InfraConfig, InfraSecrets } from "../../shared/types";
import { renderCloudInit } from "../../userdata/cloudInit";

export interface CreateComputeArgs {
  config: InfraConfig;
  secrets: InfraSecrets;
  tags: Record<string, string>;
  network: AwsNetworkOutputs;
  iam: AwsIamOutputs;
  fqdn: string;
  tunnelId: pulumi.Output<string>;
}

export function createCompute(args: CreateComputeArgs): AwsComputeOutputs {
  const ami = aws.ec2.getAmiOutput({
    owners: ["amazon"],
    mostRecent: true,
    filters: [
      {
        name: "name",
        values: ["al2023-ami-*-kernel-6.1-arm64"],
      },
      {
        name: "architecture",
        values: [args.config.instanceAmiArch],
      },
      {
        name: "root-device-type",
        values: ["ebs"],
      },
      {
        name: "virtualization-type",
        values: ["hvm"],
      },
    ],
  });

  const userData = renderCloudInit({
    fqdn: args.fqdn,
    servicePort: args.config.servicePort,
    cloudflareAccountId: args.config.cloudflareAccountId,
    tunnelId: args.tunnelId,
    tunnelSecret: args.secrets.tunnelSecret,
    surrealdbRootPassword: args.secrets.surrealdbRootPassword,
    wasmcloudVersion: args.config.wasmcloudVersion,
    surrealdbVersion: args.config.surrealdbVersion,
  });

  const instance = new aws.ec2.Instance(stackResourceName(args.config, "origin"), {
    ami: ami.id,
    instanceType: args.config.instanceType,
    subnetId: args.network.privateSubnetId,
    vpcSecurityGroupIds: [args.network.instanceSecurityGroupId],
    associatePublicIpAddress: false,
    iamInstanceProfile: args.iam.instanceProfileName,
    userData,
    userDataReplaceOnChange: true,
    metadataOptions: {
      httpEndpoint: "enabled",
      httpTokens: "required",
    },
    rootBlockDevice: {
      volumeType: "gp3",
      volumeSize: 30,
      encrypted: true,
    },
    tags: args.tags,
  });

  return {
    instanceId: instance.id,
    privateIp: instance.privateIp,
    instanceRoleArn: args.iam.instanceRoleArn,
  };
}
