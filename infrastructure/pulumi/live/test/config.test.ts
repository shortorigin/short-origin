import { describe, expect, it } from "vitest";
import { validateInfraConfig } from "../src/config/schema";

describe("validateInfraConfig", () => {
  const base = {
    projectName: "short-origin",
    env: "dev" as const,
    awsRegion: "us-west-2",
    vpcCidr: "10.40.0.0/16",
    publicSubnetCidr: "10.40.1.0/24",
    privateSubnetCidr: "10.40.10.0/24",
    instanceType: "t4g.small",
    instanceAmiArch: "arm64" as const,
    rootDomain: "example.com",
    subdomain: "dev.api",
    cloudflareAccountId: "acct",
    cloudflareZoneId: "zone",
    tunnelName: "short-origin-dev",
    servicePort: 8080,
    ssmPathPrefix: "/short-origin",
    wasmcloudVersion: "1.2.1",
    surrealdbVersion: "2.2.1",
  };

  it("accepts valid config", () => {
    const parsed = validateInfraConfig(base);
    expect(parsed.env).toBe("dev");
    expect(parsed.instanceAmiArch).toBe("arm64");
  });

  it("rejects invalid CIDR", () => {
    expect(() =>
      validateInfraConfig({
        ...base,
        vpcCidr: "invalid-cidr",
      }),
    ).toThrow();
  });

  it("rejects invalid environment", () => {
    expect(() =>
      validateInfraConfig({
        ...base,
        env: "qa" as "dev",
      }),
    ).toThrow();
  });

  it("accepts stage configuration", () => {
    const parsed = validateInfraConfig({
      ...base,
      env: "stage" as const,
      subdomain: "stage.api",
      tunnelName: "short-origin-stage",
    });

    expect(parsed.env).toBe("stage");
  });
});
