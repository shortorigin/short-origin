import { z } from "zod";

const cidrRegex = /^(?:\d{1,3}\.){3}\d{1,3}\/\d{1,2}$/;

export const infraConfigSchema = z.object({
  projectName: z.string().min(1),
  env: z.enum(["dev", "stage", "prod"]),
  awsRegion: z.string().min(1),
  vpcCidr: z.string().regex(cidrRegex, "vpcCidr must be a CIDR string"),
  publicSubnetCidr: z.string().regex(cidrRegex, "publicSubnetCidr must be a CIDR string"),
  privateSubnetCidr: z.string().regex(cidrRegex, "privateSubnetCidr must be a CIDR string"),
  instanceType: z.string().min(1),
  instanceAmiArch: z.literal("arm64"),
  rootDomain: z.string().min(1),
  subdomain: z.string().min(1),
  cloudflareAccountId: z.string().min(1),
  cloudflareZoneId: z.string().min(1),
  tunnelName: z.string().min(1),
  servicePort: z.number().int().min(1).max(65535),
  ssmPathPrefix: z.string().startsWith("/"),
  wasmcloudVersion: z.string().min(1),
  surrealdbVersion: z.string().min(1),
});

export type InfraConfigInput = z.infer<typeof infraConfigSchema>;

export function validateInfraConfig(input: InfraConfigInput): InfraConfigInput {
  return infraConfigSchema.parse(input);
}
