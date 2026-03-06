import { InfraConfig } from "./types";

export function defaultTags(config: InfraConfig): Record<string, string> {
  return {
    Project: config.projectName,
    Environment: config.env,
    ManagedBy: "pulumi",
    Stack: config.env,
  };
}
