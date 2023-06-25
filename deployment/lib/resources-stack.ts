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
import * as ecr from "aws-cdk-lib/aws-ecr";
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
import * as sqs from "aws-cdk-lib/aws-sqs";
import * as batch_alpha from "@aws-cdk/aws-batch-alpha";
import { exec, execSync } from "child_process";
import * as elb from "aws-cdk-lib/aws-elasticloadbalancingv2";
import * as iam from "aws-cdk-lib/aws-iam";
import { FckNatInstanceProvider } from "cdk-fck-nat";
import cluster from "cluster";

export class BuilderWorkerConstruct extends Construct {
  constructor(
    scope: Construct,
    id: string,
    vpc: ec2.Vpc,
    bot_s3: s3.Bucket,
    compiled_bot_s3: s3.Bucket,
    bot_uploads_sqs: sqs.Queue,
    build_results_sqs: sqs.Queue,
    cluster: ecs.Cluster
  ) {
    super(scope, id);

    const image = ecs.ContainerImage.fromDockerImageAsset(
      new DockerImageAsset(this, "api-image", {
        file: "workers/builder/Dockerfile",
        directory: "..",
        networkMode: NetworkMode.DEFAULT,
      })
    );

    const task = new ecs.FargateTaskDefinition(this, "builder-task", {
      cpu: 256,
      memoryLimitMiB: 512,

      runtimePlatform: {
        cpuArchitecture: ecs.CpuArchitecture.ARM64,
        operatingSystemFamily: ecs.OperatingSystemFamily.LINUX,
      },
    });

    const container = task.addContainer("builder-container", {
      image,
      environment: {
        BOT_S3_BUCKET: bot_s3.bucketName,
        COMPILED_BOT_S3_BUCKET: compiled_bot_s3.bucketName,
        BOT_UPLOADS_QUEUE_URL: bot_uploads_sqs.queueUrl,
        BUILD_RESULTS_QUEUE_URL: build_results_sqs.queueUrl,
      },
      logging: new ecs.AwsLogDriver({
        streamPrefix: "builder",
      }),
    });

    // TODO: restrict these permissions

    bot_s3.grantRead(task.taskRole);
    compiled_bot_s3.grantWrite(task.taskRole);

    bot_uploads_sqs.grantConsumeMessages(task.taskRole);
    build_results_sqs.grantSendMessages(task.taskRole);

    const service = new ecs.FargateService(this, "results-service", {
      cluster,
      taskDefinition: task,
    });
  }
}

export class GameplayWorkerConstruct extends Construct {
  constructor(
    scope: Construct,
    id: string,
    vpc: ec2.Vpc,
    compiled_bot_s3: s3.Bucket,
    new_games_sqs: sqs.Queue,
    game_results_sqs: sqs.Queue,
    cluster: ecs.Cluster
  ) {
    super(scope, id);

    const image = ecs.ContainerImage.fromDockerImageAsset(
      new DockerImageAsset(this, "api-image", {
        directory: "..",
        file: "workers/gameplay/Dockerfile",
        networkMode: NetworkMode.DEFAULT,
      })
    );

    const task = new ecs.FargateTaskDefinition(this, "gameplay-task", {
      cpu: 256,
      memoryLimitMiB: 512,

      runtimePlatform: {
        cpuArchitecture: ecs.CpuArchitecture.ARM64,
        operatingSystemFamily: ecs.OperatingSystemFamily.LINUX,
      },
    });

    const container = task.addContainer("gameplay-container", {
      image,
      environment: {
        COMPILED_BOT_S3_BUCKET: compiled_bot_s3.bucketName,
        GAME_RESULTS_QUEUE_URL: game_results_sqs.queueUrl,
        NEW_GAMES_QUEUE_URL: new_games_sqs.queueUrl,
      },
      logging: new ecs.AwsLogDriver({
        streamPrefix: "worker",
      }),
    });

    compiled_bot_s3.grantRead(task.taskRole);

    new_games_sqs.grantConsumeMessages(task.taskRole);
    game_results_sqs.grantSendMessages(task.taskRole);
    const service = new ecs.FargateService(this, "results-service", {
      cluster,
      taskDefinition: task,
    });
  }
}

export class ResultsWorkerConstruct extends Construct {
  constructor(
    scope: Construct,
    id: string,
    vpc: ec2.Vpc,
    password: sman.Secret,
    game_results_sqs: sqs.Queue,
    build_results_sqs: sqs.Queue,
    new_games_sqs: sqs.Queue,
    db: rds.DatabaseInstance,
    cluster: ecs.Cluster
  ) {
    super(scope, id);

    const image = ecs.ContainerImage.fromDockerImageAsset(
      new DockerImageAsset(this, "api-image", {
        directory: "..",
        file: "workers/results/Dockerfile",
        networkMode: NetworkMode.DEFAULT,
      })
    );

    const task = new ecs.FargateTaskDefinition(this, "results-task", {
      cpu: 256,
      memoryLimitMiB: 512,

      runtimePlatform: {
        cpuArchitecture: ecs.CpuArchitecture.ARM64,
        operatingSystemFamily: ecs.OperatingSystemFamily.LINUX,
      },
    });

    const container = task.addContainer("results-container", {
      image,
      environment: {
        DB_USER: "postgres",
        DB_URL: db.instanceEndpoint.socketAddress,
        GAME_RESULTS_QUEUE_URL: game_results_sqs.queueUrl,
        BUILD_RESULTS_QUEUE_URL: build_results_sqs.queueUrl,
        NEW_GAMES_QUEUE_URL: new_games_sqs.queueUrl,
      },
      secrets: {
        DB_PASSWORD: ecs.Secret.fromSecretsManager(password),
      },
      logging: new ecs.AwsLogDriver({
        streamPrefix: "results",
      }),
    });
    const service = new ecs.FargateService(this, "results-service", {
      cluster,
      taskDefinition: task,
    });

    game_results_sqs.grantConsumeMessages(task.taskRole);
    build_results_sqs.grantConsumeMessages(task.taskRole);
    new_games_sqs.grantSendMessages(task.taskRole);

    db.connections.allowDefaultPortFrom(service);
  }
}

export class ScalingAPIConstruct extends Construct {
  readonly loadBalancer: ecs_patterns.ApplicationLoadBalancedFargateService;
  readonly pfp_s3: s3.Bucket;
  constructor(
    scope: Construct,
    id: string,
    db: rds.DatabaseInstance,
    vpc: ec2.Vpc,
    password: sman.Secret,
    cert: certManager.Certificate,
    domainName: string,
    bot_s3: s3.Bucket,
    bot_uploads_sqs: sqs.Queue,
    new_games_sqs: sqs.Queue,
    cluster: ecs.Cluster
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

    const image = ecs.ContainerImage.fromDockerImageAsset(
      new DockerImageAsset(this, "api-image", {
        directory: "..",
        file: "website/Dockerfile",
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
            PFP_S3_BUCKET: this.pfp_s3.bucketName,
            BOT_S3_BUCKET: bot_s3.bucketName,
            BOT_SIZE: "5000000",
            APP_PFP_ENDPOINT: this.pfp_s3.urlForObject(),
            BOT_UPLOADS_QUEUE_URL: bot_uploads_sqs.queueUrl,
            NEW_GAMES_QUEUE_URL: new_games_sqs.queueUrl,
            RUST_LOG: "info",
            PORT: "80",
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
    this.pfp_s3.grantReadWrite(
      this.loadBalancer.service.taskDefinition.taskRole
    );
    this.pfp_s3.grantPutAcl(this.loadBalancer.service.taskDefinition.taskRole);
    this.pfp_s3.grantPut(this.loadBalancer.service.taskDefinition.taskRole);

    bot_s3.grantReadWrite(this.loadBalancer.service.taskDefinition.taskRole);

    bot_uploads_sqs.grantSendMessages(
      this.loadBalancer.service.taskDefinition.taskRole
    );
    new_games_sqs.grantSendMessages(
      this.loadBalancer.service.taskDefinition.taskRole
    );
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

    const dbPassword = new sman.Secret(this, "db-password", {
      generateSecretString: {
        excludePunctuation: true,
      },
    });
    const db = new rds.DatabaseInstance(this, "db", {
      engine: rds.DatabaseInstanceEngine.postgres({
        version: rds.PostgresEngineVersion.VER_15,
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

    const bot_s3 = new s3.Bucket(this, "bot");
    const compiled_bot_s3 = new s3.Bucket(this, "compiled-bot");

    const bot_uploads_sqs = new sqs.Queue(this, "bot-uploads");
    const new_games_sqs = new sqs.Queue(this, "new-games");
    const game_results_sqs = new sqs.Queue(this, "game-results");
    const build_results_sqs = new sqs.Queue(this, "build-results");

    const cluster = new ecs.Cluster(this, "cluster", {
      vpc,
    });

    const api = new ScalingAPIConstruct(
      this,
      "api",
      db,
      vpc,
      dbPassword,
      cert,
      process.env.APP_DOMAIN_NAME as string,
      bot_s3,
      bot_uploads_sqs,
      new_games_sqs,
      cluster
    );

    const builderWorker = new BuilderWorkerConstruct(
      this,
      "builder-worker",
      vpc,
      bot_s3,
      compiled_bot_s3,
      bot_uploads_sqs,
      build_results_sqs,
      cluster
    );
    const gameplayWorker = new GameplayWorkerConstruct(
      this,
      "gameplay-worker",
      vpc,
      compiled_bot_s3,
      new_games_sqs,
      game_results_sqs,
      cluster
    );

    const resultsWorker = new ResultsWorkerConstruct(
      this,
      "results-worker",
      vpc,
      dbPassword,
      game_results_sqs,
      build_results_sqs,
      new_games_sqs,
      db,
      cluster
    );
  }
}
