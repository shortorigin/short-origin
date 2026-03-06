import { InfraConfig } from "./types";

const MAX_AWS_NAME = 63;

export function stackResourceName(config: Pick<InfraConfig, "projectName" | "env">, base: string): string {
  const raw = `${config.projectName}-${config.env}-${base}`.toLowerCase();
  if (raw.length <= MAX_AWS_NAME) {
    return raw;
  }
  return raw.slice(0, MAX_AWS_NAME);
}

export function composeFqdn(config: Pick<InfraConfig, "subdomain" | "rootDomain">): string {
  return `${config.subdomain}.${config.rootDomain}`;
}
