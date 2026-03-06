import * as aws from "@pulumi/aws";
import * as pulumi from "@pulumi/pulumi";
import { stackResourceName } from "../../shared/naming";
import { InfraConfig } from "../../shared/types";

export interface ObservabilityArgs {
  config: InfraConfig;
  tags: Record<string, string>;
  instanceId: pulumi.Output<string>;
}

export function createObservability(args: ObservabilityArgs): void {
  new aws.cloudwatch.LogGroup(stackResourceName(args.config, "system-log-group"), {
    name: `/short-origin/${args.config.env}/system`,
    retentionInDays: 30,
    tags: args.tags,
  });

  new aws.cloudwatch.MetricAlarm(stackResourceName(args.config, "cpu-high"), {
    alarmDescription: "CPU utilization exceeds 80% for 10 minutes",
    comparisonOperator: "GreaterThanThreshold",
    evaluationPeriods: 2,
    metricName: "CPUUtilization",
    namespace: "AWS/EC2",
    period: 300,
    statistic: "Average",
    threshold: 80,
    dimensions: {
      InstanceId: args.instanceId,
    },
    tags: args.tags,
  });

  new aws.cloudwatch.MetricAlarm(stackResourceName(args.config, "status-check"), {
    alarmDescription: "EC2 instance failed status checks",
    comparisonOperator: "GreaterThanThreshold",
    evaluationPeriods: 2,
    metricName: "StatusCheckFailed",
    namespace: "AWS/EC2",
    period: 60,
    statistic: "Maximum",
    threshold: 0,
    dimensions: {
      InstanceId: args.instanceId,
    },
    tags: args.tags,
  });
}
