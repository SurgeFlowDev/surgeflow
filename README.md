# surgeflow
An experimental alternative to Cloudflare Workflows, Temporal, Airflow.

I'm aiming to have an easy, idiomatic way to write workflows in Rust, and have an easy way to deploy them to arbitrary infrastructure with all the bells and whistles: retries, caching, sleeping, human-in-the-loop events, etc..

This will be achieved through adapter crates:
- https://github.com/SurgeFlowDev/adapter-embedded: embedded, for running on a single machine. useful for development and cases where you want to rely on surgeflow's facilities but don't need the full power of a distributed system.
- https://github.com/SurgeFlowDev/adapter-aws-sqs-dynamodb-postgres: for running on AWS infrastructure, leveraging SQS, DynamoDB, and Postgres (postgres doesn't have to be on AWS).
