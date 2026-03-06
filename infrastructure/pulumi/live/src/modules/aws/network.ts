import * as aws from "@pulumi/aws";
import * as pulumi from "@pulumi/pulumi";
import { stackResourceName } from "../../shared/naming";
import { AwsNetworkOutputs, InfraConfig } from "../../shared/types";

export function buildNetworkPlan(config: Pick<InfraConfig, "vpcCidr" | "publicSubnetCidr" | "privateSubnetCidr">): {
  vpcCidr: string;
  publicSubnetCidr: string;
  privateSubnetCidr: string;
  ingressRuleCount: number;
} {
  return {
    vpcCidr: config.vpcCidr,
    publicSubnetCidr: config.publicSubnetCidr,
    privateSubnetCidr: config.privateSubnetCidr,
    ingressRuleCount: 0,
  };
}

export function createAwsNetwork(
  config: InfraConfig,
  tags: Record<string, string>,
): AwsNetworkOutputs {
  const azs = aws.getAvailabilityZonesOutput({
    state: "available",
  });
  const primaryAz = azs.names.apply((names) => names[0]);

  const vpc = new aws.ec2.Vpc(stackResourceName(config, "vpc"), {
    cidrBlock: config.vpcCidr,
    enableDnsHostnames: true,
    enableDnsSupport: true,
    tags,
  });

  const igw = new aws.ec2.InternetGateway(stackResourceName(config, "igw"), {
    vpcId: vpc.id,
    tags,
  });

  const publicSubnet = new aws.ec2.Subnet(stackResourceName(config, "public-subnet"), {
    vpcId: vpc.id,
    cidrBlock: config.publicSubnetCidr,
    mapPublicIpOnLaunch: true,
    availabilityZone: primaryAz,
    tags,
  });

  const privateSubnet = new aws.ec2.Subnet(stackResourceName(config, "private-subnet"), {
    vpcId: vpc.id,
    cidrBlock: config.privateSubnetCidr,
    mapPublicIpOnLaunch: false,
    availabilityZone: primaryAz,
    tags,
  });

  const natEip = new aws.ec2.Eip(stackResourceName(config, "nat-eip"), {
    domain: "vpc",
    tags,
  });

  const natGateway = new aws.ec2.NatGateway(stackResourceName(config, "nat"), {
    allocationId: natEip.id,
    subnetId: publicSubnet.id,
    tags,
  });

  const publicRouteTable = new aws.ec2.RouteTable(stackResourceName(config, "public-rt"), {
    vpcId: vpc.id,
    routes: [
      {
        cidrBlock: "0.0.0.0/0",
        gatewayId: igw.id,
      },
    ],
    tags,
  });

  const privateRouteTable = new aws.ec2.RouteTable(stackResourceName(config, "private-rt"), {
    vpcId: vpc.id,
    routes: [
      {
        cidrBlock: "0.0.0.0/0",
        natGatewayId: natGateway.id,
      },
    ],
    tags,
  });

  new aws.ec2.RouteTableAssociation(stackResourceName(config, "public-rta"), {
    subnetId: publicSubnet.id,
    routeTableId: publicRouteTable.id,
  });

  new aws.ec2.RouteTableAssociation(stackResourceName(config, "private-rta"), {
    subnetId: privateSubnet.id,
    routeTableId: privateRouteTable.id,
  });

  const instanceSecurityGroup = new aws.ec2.SecurityGroup(stackResourceName(config, "instance-sg"), {
    vpcId: vpc.id,
    description: "Private origin SG with no inbound ingress",
    ingress: [],
    egress: [
      {
        protocol: "-1",
        fromPort: 0,
        toPort: 0,
        cidrBlocks: ["0.0.0.0/0"],
      },
    ],
    tags,
  });

  return {
    vpcId: vpc.id,
    publicSubnetId: publicSubnet.id,
    privateSubnetId: privateSubnet.id,
    instanceSecurityGroupId: instanceSecurityGroup.id,
  };
}
