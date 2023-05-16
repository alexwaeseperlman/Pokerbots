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

    this.bucket = new s3.Bucket(this, "pfp", {
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      objectOwnership: s3.ObjectOwnership.BUCKET_OWNER_PREFERRED,
    });
    this.bucket.addCorsRule({
      allowedMethods: [s3.HttpMethods.GET, s3.HttpMethods.PUT],
      allowedOrigins: ["*"],
      allowedHeaders: ["*"],
    });
  }
}

export class BotConstruct extends Construct {
  public readonly bucket: s3.Bucket;
  public readonly onCreationLambda: lambda.Function;

  constructor(scope: Construct, id: string, vpc: ec2.Vpc) {
    super(scope, id);

    this.bucket = new s3.Bucket(this, "bot", {
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      objectOwnership: s3.ObjectOwnership.BUCKET_OWNER_ENFORCED,
    });
    this.onCreationLambda = new lambda.Function(this, "onCreationLambda", {
      runtime: lambda.Runtime.NODEJS_18_X,
      handler: "index.handler",
      code: lambda.Code.fromAsset("../lambdas/bot-uploaded"),
      environment: {
        BOT_SIZE: process.env.BOT_SIZE ?? "1000000",
      },
    });
    this.bucket.addObjectCreatedNotification(
      new s3_notify.LambdaDestination(this.onCreationLambda),
      {
        prefix: "upload/",
      }
    );

    // batch queue for building bots
    const env = new batch_alpha.FargateComputeEnvironment(this, "batch-env", {
      vpc,
      spot: true,
    });

    /*const buildImage = ecs.ContainerImage.fromDockerImageAsset(
      new DockerImageAsset(this, "bots-container", {
        directory: "../bots/buildenv",
      })
    );*/
    const playImage = ecs.ContainerImage.fromDockerImageAsset(
      new DockerImageAsset(this, "bots-container", {
        directory: "../bots",
      })
    );
    /*const buildJobDefn = new batch_alpha.EcsJobDefinition(this, "build-job", {
      container: new batch_alpha.EcsFargateContainerDefinition(
        this,
        "build-container",
        {
          cpu: 256,
          memory: cdk.Size.mebibytes(512),
          image: buildImage,
        }
      ),
    });*/
    const playJobDefn = new batch_alpha.EcsJobDefinition(this, "play-job", {
      container: new batch_alpha.EcsFargateContainerDefinition(
        this,
        "play-container",
        {
          cpu: 0.25,
          memory: cdk.Size.mebibytes(512),
          image: playImage,
          command: ["Ref::botA", "Ref::botB", "Ref::resultUrl"],
          logging: new ecs.AwsLogDriver({
            streamPrefix: "play",
            logRetention: cdk.aws_logs.RetentionDays.ONE_DAY,
          }),
        }
      ),
    });
    const queue = new batch_alpha.JobQueue(this, "queue", {
      computeEnvironments: [
        {
          computeEnvironment: env,
          order: 1,
        },
      ],
    });
  }
}

export class ScalingAPIConstruct extends Construct {
  readonly loadBalancer: ecs_patterns.ApplicationLoadBalancedFargateService;
  constructor(
    scope: Construct,
    id: string,
    pfp_s3_bucket: s3.Bucket,
    db: rds.DatabaseInstance,
    vpc: ec2.Vpc,
    password: sman.Secret,
    cert: certManager.Certificate,
    domainName: string,
    bots: BotConstruct
  ) {
    super(scope, id);

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
            APP_PFP_S3_BUCKET: pfp_s3_bucket.bucketName,
            REDIRECT_URI: `https://${domainName}/api/login`,
            PORT: "80",
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
    bots.bucket.grantPut(this.loadBalancer.service.taskDefinition.taskRole);

    new cdk.CfnOutput(this, "api-url", {
      value: this.loadBalancer.loadBalancer.loadBalancerDnsName,
    });
    new cdk.CfnOutput(this, "api-task-role", {
      value: this.loadBalancer.service.taskDefinition.taskRole.roleArn,
    });
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
    const bots = new BotConstruct(this, "bots", vpc);

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
      process.env.APP_DOMAIN_NAME as string,
      bots
    );
    /*const cf = new cloudfront.Distribution(this, "cdnDistribution", {
      defaultBehavior: {
        origin: new origins.LoadBalancerV2Origin(api.loadBalancer.loadBalancer),
        viewerProtocolPolicy: cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
      },
      additionalBehaviors: {
        "/api/*": {
          origin: new origins.LoadBalancerV2Origin(
            api.loadBalancer.loadBalancer
          ),
          viewerProtocolPolicy:
            cloudfront.ViewerProtocolPolicy.REDIRECT_TO_HTTPS,
          cachePolicy: cloudfront.CachePolicy.CACHING_DISABLED,
        },
      },
      domainNames: [process.env.APP_DOMAIN_NAME as string],
      certificate: cert,
    });*/
    /*api.loadBalancer.taskDefinition.defaultContainer?.addEnvironment(
      "REDIRECT_URI",
      `https://${cf.domainName}/api/login`
    );*/

    /*cf.addBehavior("/api/*", new origins.LoadBalancerV2Origin(
        api.service.loadBalancer.loadBalancer
      ))*/

    /*new route53.ARecord(this, "CDNARecord", {
      zone,
      target: route53.RecordTarget.fromAlias(new targets.CloudFrontTarget(cf)),
    });

    new route53.AaaaRecord(this, "AliasRecord", {
      zone,
      target: route53.RecordTarget.fromAlias(new targets.CloudFrontTarget(cf)),
    });*/
    // build the frontend and upload it to s3
  }
}
