import * as cloudflare from "@pulumi/cloudflare";
import { stackResourceName } from "../../shared/naming";
import { InfraConfig } from "../../shared/types";

export function createEdgeSecurityBaseline(config: InfraConfig): void {
  new cloudflare.ZoneSetting(stackResourceName(config, "ssl-setting"), {
    zoneId: config.cloudflareZoneId,
    settingId: "ssl",
    value: "strict",
  });

  new cloudflare.ZoneSetting(stackResourceName(config, "https-setting"), {
    zoneId: config.cloudflareZoneId,
    settingId: "always_use_https",
    value: "on",
  });

  new cloudflare.ZoneSetting(stackResourceName(config, "min-tls-setting"), {
    zoneId: config.cloudflareZoneId,
    settingId: "min_tls_version",
    value: "1.2",
  });

  new cloudflare.Ruleset(stackResourceName(config, "waf-baseline"), {
    zoneId: config.cloudflareZoneId,
    name: `${config.projectName}-${config.env}-waf-baseline`,
    description: "Baseline WAF custom rules for short origin",
    kind: "zone",
    phase: "http_request_firewall_custom",
    rules: [
      {
        action: "managed_challenge",
        expression: "not http.request.method in {\"GET\" \"HEAD\" \"POST\" \"OPTIONS\"}",
        description: "Challenge uncommon HTTP methods",
        enabled: true,
      },
    ],
  });
}
