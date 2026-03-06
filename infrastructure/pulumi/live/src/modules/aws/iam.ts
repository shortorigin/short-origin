import * as aws from "@pulumi/aws";
import { stackResourceName } from "../../shared/naming";
import { AwsIamOutputs, InfraConfig } from "../../shared/types";

export function createEc2Iam(
  config: InfraConfig,
  tags: Record<string, string>,
): AwsIamOutputs {
  const role = new aws.iam.Role(stackResourceName(config, "ec2-role"), {
    assumeRolePolicy: aws.iam.assumeRolePolicyForPrincipal({
      Service: "ec2.amazonaws.com",
    }),
    tags,
  });

  new aws.iam.RolePolicyAttachment(stackResourceName(config, "ec2-role-ssm"), {
    role: role.name,
    policyArn: "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore",
  });

  new aws.iam.RolePolicyAttachment(stackResourceName(config, "ec2-role-cw"), {
    role: role.name,
    policyArn: "arn:aws:iam::aws:policy/CloudWatchAgentServerPolicy",
  });

  const ssmReadPolicy = new aws.iam.RolePolicy(stackResourceName(config, "ec2-role-ssm-read"), {
    role: role.id,
    policy: {
      Version: "2012-10-17",
      Statement: [
        {
          Effect: "Allow",
          Action: [
            "ssm:GetParameter",
            "ssm:GetParameters",
            "ssm:GetParametersByPath",
          ],
          Resource: `arn:aws:ssm:${config.awsRegion}:*:parameter${config.ssmPathPrefix}/*`,
        },
      ],
    },
  });

  const instanceProfile = new aws.iam.InstanceProfile(stackResourceName(config, "ec2-profile"), {
    role: role.name,
    tags,
  }, { dependsOn: [ssmReadPolicy] });

  return {
    instanceProfileName: instanceProfile.name,
    instanceRoleArn: role.arn,
  };
}
