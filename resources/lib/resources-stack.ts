import * as dotenv from "dotenv";
dotenv.config({
  path: "../.env",
});
import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";
import * as s3 from "aws-cdk-lib/aws-s3";
import * as certManager from "aws-cdk-lib/aws-certificatemanager";
import * as cloudfront from "aws-cdk-lib/aws-cloudfront";
import * as origins from "aws-cdk-lib/aws-cloudfront-origins";
import * as route53 from "aws-cdk-lib/aws-route53";
import * as targets from "aws-cdk-lib/aws-route53-targets";
import * as ecs from "aws-cdk-lib/aws-ecs";
import * as ec2 from "aws-cdk-lib/aws-ec2";
import * as ecs_patterns from "aws-cdk-lib/aws-ecs-patterns";
import { DockerImageAsset, NetworkMode } from "aws-cdk-lib/aws-ecr-assets";
import * as rds from "aws-cdk-lib/aws-rds";
import * as crypto from "crypto";
import * as sman from "aws-cdk-lib/aws-secretsmanager";
import * as ssm from "aws-cdk-lib/aws-ssm";
import * as s3deploy from "aws-cdk-lib/aws-s3-deployment";
import * as pipelines from "aws-cdk-lib/pipelines";
import * as s3_notify from "aws-cdk-lib/aws-s3-notifications";
import * as lambda from "aws-cdk-lib/aws-lambda";
import * as batch from "aws-cdk-lib/aws-batch";
import * as batch_alpha from "@aws-cdk/aws-batch-alpha";
import { exec, execSync } from "child_process";
import * as elb from "aws-cdk-lib/aws-elasticloadbalancingv2";
import * as iam from "aws-cdk-lib/aws-iam";
import { FckNatInstanceProvider } from "cdk-fck-nat";

export class PFPS3Construct extends Construct {
  public readonly bucket: s3.Bucket;

  constructor(scope: Construct, id: string) {
    super(scope, id);
  }
}

export class ScalingAPIConstruct extends Construct {
  readonly loadBalancer: ecs_patterns.ApplicationLoadBalancedFargateService;
  readonly pfp_s3: s3.Bucket;
  constructor(
    scope: Construct,
    id: string,
    pfp_s3_bucket: s3.Bucket,
    db: rds.DatabaseInstance,
    vpc: ec2.Vpc,
    password: sman.Secret,
    cert: certManager.Certificate,
    domainName: string
  ) {
    super(scope, id);

    this.pfp_s3 = new s3.Bucket(this, "pfp", {
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      objectOwnership: s3.ObjectOwnership.BUCKET_OWNER_PREFERRED,
    });
    this.pfp_s3.addCorsRule({
      allowedMethods: [s3.HttpMethods.GET, s3.HttpMethods.PUT],
      allowedOrigins: ["*"],
      allowedHeaders: ["*"],
    });

    const cluster = new ecs.Cluster(this, "api-cluster", {
      vpc,
    });

    const image = ecs.ContainerImage.fromDockerImageAsset(
      new DockerImageAsset(this, "api-image", {
        directory: "../website",
        networkMode: NetworkMode.HOST,
      })
    );
    const secretKey = ecs.Secret.fromSecretsManager(
      new sman.Secret(this, "secret-key-session", {
        generateSecretString: {
          passwordLength: 96,
        },
      })
    );

    this.loadBalancer = new ecs_patterns.ApplicationLoadBalancedFargateService(
      this,
      "backend-service",
      {
        cluster,
        circuitBreaker: {
          rollback: true,
        },
        runtimePlatform: {
          cpuArchitecture: ecs.CpuArchitecture.ARM64,
          operatingSystemFamily: ecs.OperatingSystemFamily.LINUX,
        },
        redirectHTTP: true,
        targetProtocol: elb.ApplicationProtocol.HTTP,
        taskImageOptions: {
          image,

          environment: {
            DB_USER: "postgres",
            DB_URL: db.instanceEndpoint.socketAddress,
            MICROSOFT_CLIENT_ID: process.env.MICROSOFT_CLIENT_ID ?? "",
            MICROSOFT_TENANT_ID: process.env.MICROSOFT_TENANT_ID ?? "",
            PFP_S3_BUCKET: pfp_s3_bucket.bucketName,
            REDIRECT_URI: `https://${domainName}/api/login`,
          },
          secrets: {
            SECRET_KEY: secretKey,
            DB_PASSWORD: ecs.Secret.fromSecretsManager(password),
            AZURE_SECRET: ecs.Secret.fromSecretsManager(
              new sman.Secret(this, "azure-secret", {
                secretStringValue: cdk.SecretValue.unsafePlainText(
                  process.env.AZURE_SECRET ?? ""
                ),
              })
            ),
          }, //*/
        },
        protocol: elb.ApplicationProtocol.HTTPS,
        certificate: cert,
      }
    );
    //db.grantConnect(this.loadBalancer.service.taskDefinition.taskRole);
    db.connections.allowDefaultPortFrom(this.loadBalancer.service);
    pfp_s3_bucket.grantReadWrite(
      this.loadBalancer.service.taskDefinition.taskRole
    );
    pfp_s3_bucket.grantPutAcl(
      this.loadBalancer.service.taskDefinition.taskRole
    );
    pfp_s3_bucket.grantPut(this.loadBalancer.service.taskDefinition.taskRole);
  }
}

export class ResourcesStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    /*const zone = route53.HostedZone.fromHostedZoneAttributes(this, "zone", {
      hostedZoneId: "us-east-1",
      zoneName: process.env.APP_DOMAIN_NAME as string,
    });*/
    const cert = new certManager.Certificate(this, "cert", {
      domainName: process.env.APP_DOMAIN_NAME as string,
      validation: certManager.CertificateValidation.fromDns(),
    });
    const vpc = new ec2.Vpc(this, "app-vpc", {
      maxAzs: 3,
      natGatewayProvider: ec2.NatProvider.instance({
        instanceType: ec2.InstanceType.of(
          ec2.InstanceClass.T3A,
          ec2.InstanceSize.MICRO
        ),
      }),
      natGateways: 1,
    });
    const pfp_s3 = new PFPS3Construct(this, "pfp_s3");

    const dbPassword = new sman.Secret(this, "db-password", {
      generateSecretString: {
        excludePunctuation: true,
      },
    });
    const db = new rds.DatabaseInstance(this, "db", {
      engine: rds.DatabaseInstanceEngine.postgres({
        version: rds.PostgresEngineVersion.VER_13_3,
      }),
      vpc,
      instanceType: ec2.InstanceType.of(
        ec2.InstanceClass.T4G,
        ec2.InstanceSize.MICRO
      ),
      credentials: {
        username: "postgres",
        password: dbPassword.secretValue,
      },
    });

    const api = new ScalingAPIConstruct(
      this,
      "api",
      pfp_s3.bucket,
      db,
      vpc,
      dbPassword,
      cert,
      process.env.APP_DOMAIN_NAME as string
    );
  }
}
