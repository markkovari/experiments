import * as pulumi from "@pulumi/pulumi";
import * as digitalocean from "@pulumi/digitalocean";

const config = new pulumi.Config();
const doToken = config.require("doToken");
const region = digitalocean.Region.FRA1;

// Create a web server
const projectName = "DEV-ENV" as const;

const project = new digitalocean.Project(projectName, { environment: "development", })

const vpcCidr = "10.20.0.0/24";

const vpc = new digitalocean.Vpc("nats-vpc", {
	region,
	ipRange: vpcCidr,
});

const droplet = new digitalocean.Droplet("nats-droplet", {
	image: "docker-20-04",
	region,
	size: digitalocean.DropletSlug.DropletS1VCPU1GB,
	vpcUuid: vpc.id,
	userData: `#cloud-config
	runcmd:
	  - docker run -d --name nats \
	      -p 4222:4222 \
	      nats:latest \
	      -js
	`,
});

const postgres_database = new digitalocean.DatabaseCluster("postgres-example", {
	name: "example-postgres-cluster",
	engine: "pg",
	version: "17",
	size: digitalocean.DatabaseSlug.DB_1VPCU1GB,
	region: digitalocean.Region.NYC1,
	nodeCount: 1,
});

const assign = new digitalocean.ProjectResources("assign-resources", {
	project: project.name,
	resources: [droplet.dropletUrn, postgres_database.urn],
});

// const firewall = new digitalocean.Firewall("nats-firewall", {
// 	dropletIds: [droplet.id],
// 	inboundRules: [
// 		{
// 			protocol: "tcp",
// 			portRange: "4222",
// 			sourceAddresses: [vpcCidr], // only VPC access
// 		},
// 		{
// 			protocol: "tcp",
// 			portRange: "22",
// 			sourceAddresses: ["0.0.0.0/0"], // SSH access
// 		},
// 	],
// 	outboundRules: [
// 		{
// 			protocol: "tcp",
// 			portRange: "all",
// 			destinationAddresses: ["0.0.0.0/0"],
// 		},
// 		{
// 			protocol: "udp",
// 			portRange: "all",
// 			destinationAddresses: ["0.0.0.0/0"],
// 		},
// 	],
// });

// const mainApp = new digitalocean.App("autorank", {
// 	projectId: project.id,
// 	spec: {
// 		name: "hello-yolo",
// 		envs: [
// 			{
// 				key: "almafa",
// 				value: config.get("sigma")
// 			},
// 		],
// 		services: [
// 			{
// 				name: 
// 			}
// 		]
// 	},
// })
