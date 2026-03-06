import { describe, expect, it } from "vitest";
import { buildNetworkPlan } from "../src/modules/aws/network";

describe("buildNetworkPlan", () => {
  it("keeps ingress rule count at zero for private origin", () => {
    const plan = buildNetworkPlan({
      vpcCidr: "10.40.0.0/16",
      publicSubnetCidr: "10.40.1.0/24",
      privateSubnetCidr: "10.40.10.0/24",
    });

    expect(plan.ingressRuleCount).toBe(0);
    expect(plan.privateSubnetCidr).toBe("10.40.10.0/24");
  });
});
