import * as aws from "@pulumi/aws";
import * as pulumi from "@pulumi/pulumi";
import * as random from "@pulumi/random";

const config = new pulumi.Config();
const stateBucketPrefix = config.get("stateBucketPrefix") ?? "short-origin-pulumi-state";
const lockTableName = config.get("lockTableName") ?? "short-origin-pulumi-locks";

const suffix = new random.RandomId("state-bucket-suffix", {
  byteLength: 4,
});

const stateBucket = new aws.s3.Bucket("pulumi-state-bucket", {
  bucket: pulumi.interpolate`${stateBucketPrefix}-${suffix.hex}`,
  tags: {
    Project: "short-origin",
    ManagedBy: "pulumi",
    Purpose: "state-backend",
  },
});

new aws.s3.BucketVersioningV2("pulumi-state-versioning", {
  bucket: stateBucket.id,
  versioningConfiguration: {
    status: "Enabled",
  },
});

new aws.s3.BucketServerSideEncryptionConfigurationV2("pulumi-state-sse", {
  bucket: stateBucket.id,
  rules: [
    {
      applyServerSideEncryptionByDefault: {
        sseAlgorithm: "AES256",
      },
    },
  ],
});

new aws.s3.BucketPublicAccessBlock("pulumi-state-public-access-block", {
  bucket: stateBucket.id,
  blockPublicAcls: true,
  blockPublicPolicy: true,
  ignorePublicAcls: true,
  restrictPublicBuckets: true,
});

const lockTable = new aws.dynamodb.Table("pulumi-lock-table", {
  name: lockTableName,
  billingMode: "PAY_PER_REQUEST",
  hashKey: "LockID",
  attributes: [
    {
      name: "LockID",
      type: "S",
    },
  ],
  tags: {
    Project: "short-origin",
    ManagedBy: "pulumi",
    Purpose: "state-locks",
  },
});

export const stateBucketName = stateBucket.bucket;
export const stateLockTableName = lockTable.name;
export const backendUrl = pulumi.interpolate`s3://${stateBucket.bucket}?region=${aws.config.region ?? "us-west-2"}&awssdk=v2`;
