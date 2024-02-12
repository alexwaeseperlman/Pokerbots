import * as dotenv from "dotenv";
dotenv.config({
  path: "../.env",
});
import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";
import * as s3 from "aws-cdk-lib/aws-s3";
import * as certManager from "aws-cdk-lib/aws-certificatemanager";
import * as ecs from "aws-cdk-lib/aws-ecs";
import * as ec2 from "aws-cdk-lib/aws-ec2";
import * as ecs_patterns from "aws-cdk-lib/aws-ecs-patterns";
import { DockerImageAsset, NetworkMode } from "aws-cdk-lib/aws-ecr-assets";
import * as rds from "aws-cdk-lib/aws-rds";
import * as sman from "aws-cdk-lib/aws-secretsmanager";
import * as sqs from "aws-cdk-lib/aws-sqs";
import * as elb from "aws-cdk-lib/aws-elasticloadbalancingv2";
import * as cloudwatch from "aws-cdk-lib/aws-cloudwatch";
import { AdjustmentType } from "aws-cdk-lib/aws-autoscaling";

export class BuilderWorkerConstruct extends Construct {
  constructor(
    scope: Construct,
    id: string,
    vpc: ec2.Vpc,
    bot_s3: s3.Bucket,
    compiled_bot_s3: s3.Bucket,
    bot_uploads_sqs: sqs.Queue,
    build_results_sqs: sqs.Queue,
    cluster: ecs.Cluster,
    build_logs_s3: s3.Bucket
  ) {
    super(scope, id);

    const image = ecs.ContainerImage.fromDockerImageAsset(
      new DockerImageAsset(this, "api-image", {
        file: "workers/builder/Dockerfile",
        directory: "..",
        networkMode: NetworkMode.DEFAULT,
      })
    );

    const task = new ecs.Ec2TaskDefinition(this, "builder-task", {
      networkMode: ecs.NetworkMode.HOST,
    });

    const container = task.addContainer("builder-container", {
      cpu: 512,
      memoryLimitMiB: 1024,
      image,
      privileged: true,
      environment: {
        BOT_S3_BUCKET: bot_s3.bucketName,
        COMPILED_BOT_S3_BUCKET: compiled_bot_s3.bucketName,
        BOT_UPLOADS_QUEUE_URL: bot_uploads_sqs.queueUrl,
        BUILD_RESULTS_QUEUE_URL: build_results_sqs.queueUrl,
        BUILD_LOGS_S3_BUCKET: build_logs_s3.bucketName,
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

    const service = new ecs.Ec2Service(this, "builder-service", {
      cluster,
      taskDefinition: task,
      desiredCount: 0,
    });

    service
      .autoScaleTaskCount({
        minCapacity: 0,
        maxCapacity: 1,
      })
      .scaleOnMetric("bot-queue-size", {
        metric: bot_uploads_sqs.metricApproximateNumberOfMessagesVisible(),
        scalingSteps: [
          { upper: 0, change: 0 },
          { lower: 1, change: 1 },
        ],
        adjustmentType: AdjustmentType.EXACT_CAPACITY,
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
    cluster: ecs.Cluster,
    game_logs_s3: s3.Bucket
  ) {
    super(scope, id);

    const image = ecs.ContainerImage.fromDockerImageAsset(
      new DockerImageAsset(this, "api-image", {
        directory: "..",
        file: "workers/gameplay/Dockerfile",
        networkMode: NetworkMode.DEFAULT,
      })
    );

    const task = new ecs.Ec2TaskDefinition(this, "gameplay-task", {
      networkMode: ecs.NetworkMode.HOST,
    });

    const container = task.addContainer("gameplay-container", {
      image,
      cpu: 256,
      memoryLimitMiB: 512,
      privileged: true,
      environment: {
        COMPILED_BOT_S3_BUCKET: compiled_bot_s3.bucketName,
        GAME_RESULTS_QUEUE_URL: game_results_sqs.queueUrl,
        NEW_GAMES_QUEUE_URL: new_games_sqs.queueUrl,
        GAME_LOGS_S3_BUCKET: game_logs_s3.bucketName,
      },
      logging: new ecs.AwsLogDriver({
        streamPrefix: "worker",
      }),
    });

    compiled_bot_s3.grantRead(task.taskRole);

    new_games_sqs.grantConsumeMessages(task.taskRole);
    game_results_sqs.grantSendMessages(task.taskRole);
    const service = new ecs.Ec2Service(this, "results-service", {
      cluster,
      taskDefinition: task,
      desiredCount: 0,
    });

    service
      .autoScaleTaskCount({
        minCapacity: 0,
        maxCapacity: 1,
      })
      .scaleOnMetric("game-queue-size", {
        metric: new_games_sqs.metricApproximateNumberOfMessagesVisible(),
        scalingSteps: [
          { upper: 0, change: 0 },
          { lower: 1, change: 1 },
        ],
        adjustmentType: AdjustmentType.EXACT_CAPACITY,
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
    cluster: ecs.Cluster,
    build_logs_s3: s3.Bucket,
    game_logs_s3: s3.Bucket
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
        BUILD_LOGS_S3_BUCKET: build_logs_s3.bucketName,
        GAME_LOGS_S3_BUCKET: game_logs_s3.bucketName,
        RUST_LOG: "info",
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
      desiredCount: 1,
    });

    game_results_sqs.grantConsumeMessages(task.taskRole);
    build_results_sqs.grantConsumeMessages(task.taskRole);
    new_games_sqs.grantSendMessages(task.taskRole);

    db.connections.allowDefaultPortFrom(service);

    game_logs_s3.grantReadWrite(task.taskRole);
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
    cluster: ecs.Cluster,
    build_logs_s3: s3.Bucket,
    game_logs_s3: s3.Bucket,
    resume_s3: s3.Bucket
  ) {
    super(scope, id);

    this.pfp_s3 = new s3.Bucket(this, "pfp", {
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      objectOwnership: s3.ObjectOwnership.BUCKET_OWNER_PREFERRED,
    });
    //this.pfp_s3.grantPublicAccess();
    this.pfp_s3.addCorsRule({
      allowedMethods: [s3.HttpMethods.GET],
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

            APP_MICROSOFT_CLIENT_ID: process.env.APP_MICROSOFT_CLIENT_ID ?? "",
            APP_GOOGLE_CLIENT_ID: process.env.APP_GOOGLE_CLIENT_ID ?? "",

            PFP_S3_BUCKET: this.pfp_s3.bucketName,
            BOT_S3_BUCKET: bot_s3.bucketName,
            GAME_LOGS_S3_BUCKET: game_logs_s3.bucketName,
            BUILD_LOGS_S3_BUCKET: build_logs_s3.bucketName,
            RESUME_S3_BUCKET: resume_s3.bucketName,
            
            BOT_SIZE: "5000000",

            BOT_UPLOADS_QUEUE_URL: bot_uploads_sqs.queueUrl,
            NEW_GAMES_QUEUE_URL: new_games_sqs.queueUrl,
            RUST_LOG: "info",
            PORT: "80",

            EMAIL_ADDRESS: process.env.EMAIL_ADDRESS ?? "",
            EMAIL_APP_PASSWORD: process.env.EMAIL_APP_PASSWORD ?? "",
            DOMAIN_NAME: domainName,
            SMTP_SERVER: process.env.SMTP_SERVER ?? "",
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
            GOOGLE_SECRET: ecs.Secret.fromSecretsManager(
              new sman.Secret(this, "google-secret", {
                secretStringValue: cdk.SecretValue.unsafePlainText(
                  process.env.GOOGLE_SECRET ?? ""
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

    build_logs_s3.grantReadWrite(
      this.loadBalancer.service.taskDefinition.taskRole
    );
    game_logs_s3.grantReadWrite(
      this.loadBalancer.service.taskDefinition.taskRole
    );

    bot_s3.grantReadWrite(this.loadBalancer.service.taskDefinition.taskRole);

    bot_uploads_sqs.grantSendMessages(
      this.loadBalancer.service.taskDefinition.taskRole
    );
    new_games_sqs.grantSendMessages(
      this.loadBalancer.service.taskDefinition.taskRole
    );

    resume_s3.grantReadWrite(this.loadBalancer.service.taskDefinition.taskRole);
  }
}

export class ResourcesStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    /*const zone = route53.HostedZone.fromHostedZoneAttributes(this, "zone", {
      hostedZoneId: "us-east-1",
      zoneName: process.env.DOMAIN_NAME as string,
    });*/
    const cert = new certManager.Certificate(this, "cert", {
      domainName: "upac.dev",
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
    const build_logs_s3 = new s3.Bucket(this, "build-logs");
    const game_logs_s3 = new s3.Bucket(this, "game-logs");
    const resume_s3 = new s3.Bucket(this, "resuems");

    const bot_uploads_sqs = new sqs.Queue(this, "bot-uploads");
    const new_games_sqs = new sqs.Queue(this, "new-games");
    const game_results_sqs = new sqs.Queue(this, "game-results");
    const build_results_sqs = new sqs.Queue(this, "build-results");

    const cluster = new ecs.Cluster(this, "cluster", {
      vpc,
    });

    const workerCluster = new ecs.Cluster(this, "unsafe-worker-cluster", {
      vpc,
    });

    const autoscalingGroup = workerCluster.addCapacity(
      "unsafe-worker-capacity",
      {
        instanceType: ec2.InstanceType.of(
          ec2.InstanceClass.C6G,
          ec2.InstanceSize.MEDIUM
        ),
        machineImage: ecs.EcsOptimizedImage.amazonLinux2(
          ecs.AmiHardwareType.ARM
        ),
        minCapacity: 0,
        maxCapacity: 1,
      }
    );

    const workerQueueMetric = new cloudwatch.MathExpression({
      expression: "m1 + m2",
      usingMetrics: {
        m1: bot_uploads_sqs.metricApproximateNumberOfMessagesVisible(),
        m2: new_games_sqs.metricApproximateNumberOfMessagesVisible(),
      },
    });
    autoscalingGroup.scaleOnMetric("worker-queue-length", {
      metric: workerQueueMetric,
      scalingSteps: [
        { upper: 0, change: 0 },
        { lower: 1, change: 1 },
      ],
      adjustmentType: AdjustmentType.EXACT_CAPACITY,
      cooldown: cdk.Duration.minutes(1),
    });
    const api = new ScalingAPIConstruct(
      this,
      "api",
      db,
      vpc,
      dbPassword,
      cert,
      "upac.dev",
      bot_s3,
      bot_uploads_sqs,
      new_games_sqs,
      cluster,
      build_logs_s3,
      game_logs_s3,
      resume_s3
    );

    const builderWorker = new BuilderWorkerConstruct(
      this,
      "builder-worker",
      vpc,
      bot_s3,
      compiled_bot_s3,
      bot_uploads_sqs,
      build_results_sqs,
      workerCluster,
      build_logs_s3
    );
    const gameplayWorker = new GameplayWorkerConstruct(
      this,
      "gameplay-worker",
      vpc,
      compiled_bot_s3,
      new_games_sqs,
      game_results_sqs,
      workerCluster,
      game_logs_s3
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
      cluster,
      build_logs_s3,
      game_logs_s3
    );
  }
}
