import { describe, expect, it } from "vitest";
import { composeFqdn } from "../src/shared/naming";

describe("composeFqdn", () => {
  it("builds environment-qualified hostname", () => {
    expect(
      composeFqdn({
        subdomain: "dev.api",
        rootDomain: "example.com",
      }),
    ).toBe("dev.api.example.com");
  });
});
