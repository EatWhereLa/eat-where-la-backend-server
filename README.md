# eat-where-la-backend-server

---

## Setting up the project repository on local machine

1. Make sure you have rust installed in your local machine
2. Run this command to install rust `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Setting up the project on docker

1. Run this command to build the project image on your local docker container
2. Make sure you are in the root directory of this repository
3. Copy this command and paste in your terminal `docker build -t eat-where-la-backend:latest .`

### Make sure to populate the `.env` file based on the `.env.example` provided in the repository

```
RUST_LOG=info
ENVIRONMENT=Development

POSTGRES_URL=postgresql://username:password@host:5432/postgres

GOOGLE_MAPS_API_BASE_URL=https://maps.google.com/maps/api/geocode/json
GOOGLE_API_KEY=<your-api-key>
```

# Setting up the application to be hosted on AWS APPRUNNER
1. Make sure you have an AWS account
2. Set up an ECR registry on AWS
3. Populate the file in directory `.github.workflows.aws.yml` with your ECR registry name and your application container name
4. The CD part of the repository is already setup through the `aws.yml` file
5. Upon a successful push to main branch in this repo, the GitHub action will build the docker image and publish it to the ECR stated in the `aws.yml` file
6. Login to the AWS Console with your AWS account credentials
7. Follow the guidelines of deploying an application on AWS APPRUNNER