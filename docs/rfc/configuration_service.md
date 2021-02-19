# Front Matter

```
Title: Configuration service
Author: Łukasz Biel
Team: CDL
Reviewer: CDLTeam
Created on: 5/2/2021
Last updated: 19/2/2021
Tracking issue: https://github.com/epiphany-platform/CommonDataLayer/issues/224
```

# Introduction

Right now we are storing metadata about repositories along schema definitions in `schema-registry`.

## Summary

We should split `schema-registry` into two separate services, one of which would serve as literal `schema-registry`,
containing only schemas and views, and other that would serve all configuration related to repositories.

## Glossary

> }|-- denotes one-to-many connection

`CS` names `command-service` \
`DR` names `data-router` \
`QS` names `query-service` \
`QR` names `query-router` \
`SR` names `schema-registry` 

## Context

We need to split business logic related to schema-registry and configuration

## Goals and Requirements

* `configuration-service` binary and docker image.
* Addition of `configuration-service` to helm and docker-compose.
* Tests for `configuration-service`.
 
no prior requirements

# Solutions

## Existing solution

Currently all information about schema/repository metadata is stored in `schema-registry`, and queried from there by `DR` and `QR`.
Schemas are cached by `DR`.

Currently only one query fetches both schema and metadata.

System isn't elastic and all configuration lies within k8s deployment files.
Schema-less environments (eg. no validator) still rely on schema-registry.

## Proposed solution

### Schema registry
`schema-registry` would serve only schemas and views. It's structure would be changed as follows:

* `schema_id` }|-- `schema_definition(version, json_schema, repository_type)`
* `view_id` }|-- `view_definition(version, jmespath)`
* `view_definition` }|-- `(schema_id, version)`


`repository_type` refers to distinction between `TIMESERIES` and `DOCUMENT_STORAGE`, 
and is necessary when validating payloads, as different kinds of repositories have slightly different message formats.

`schema-registry` must also contain repository mapping information. We will use simple key-value table,

`schema_id` -> `repository_id`

Schema must start accepting new routes, namely registration and removal of a repository, from now on.

### Configuration Service
`configuration-service` would replace `SR` functionality of serving repository metadata.

On startup, all CDL services query `configuration service` for config using `CONFIGURATION_URL.` 
Repository services would identify themselves using `repository_id.`
`repository_id` is not unique and could be shared between multiple replicas of given service. 
`command-service` and `query-service` belonging to one repository should share one `repository_id.`

The final goal is to move most environment variables away from CDL services into a static,
global database - `configuration service.`:

* Global config:
    * communication method
* Data router config:
    * Message broker
    * Input topic
    * Broker group id
    * Error reporting channel
    * Cache capacity
    * Parallel task limit
    * Monotasking
* Query router config:
    * Cache capacity
    * Input port
* Command service config (depending on communication method global):
    * Message brokers
    * Ordered input topics
    * Unordered topics
    * Consumer group_id/tag
    * Grpc port
    * Parallel task limit
    * Database connection params (We should we fine if we kept these in repos though)
* Query service (both):
    * Input port
    * Database connection params (Same as above)
    
This would leave `data-router` and `query-router` with variables:
* CONFIGURATION_URL
* SCHEMA_REGISTRY_URL

Same goes for `command-service` and `query-service`, with addition of `REPOSITORY_ID`.

All configuration in `configuration-service` should be backed by some database 
(we can consider sled here, but we should implement a wrapper over it, so features like replication are shared between `configuration-service` and `schema-registry`).
It needs to support replication and disk backups.

`configuration-service` should support `--import` and `--export` flags as current `schema-registry` does.

All communication with `configuration-service` should be done via `gRPC`,
while `API` crate should provide end-user, easy to read/write, http api access.

Supported `gRPC` queries are:

* get_config - returns config for given app
* list_repositories
* describe_repository
* add_repository
* rm_repository
* update_repository_X, where X is a repository's field

### Initial Implementation

Initial implementation of `configuration-service` should do as follows:
* move `topic`, `query-address` and `repository-type` from `SR` to `conf-service`
* add `schema_id` -> `repository_id` matcher in `SR`
* alter `DR` and `QR` to use `configuration-service` for routing

### Other considerations
#### Connector
We could write `configuration-service` as a connector, not a database. In such case CS would have eg. topic configuration provided via envs and
it would send these to `configuration-service`. `configuration-service` then would serve this metadata to DR and QR.

#### Propagate protocol via configuration service
`configuration-service` could be set up with a variable denoting protocol used in `CDL` deployment and serve it to repositories,
instead of this protocol being served to them via an env.

#### A library
We could write `configuration-service` in such way so that DR/CDLite/our clients can include it as an library.

However, using `config-service` as a library for existing CDL components is a questionable topic.

#### Propagate logging level
We could init logging level via `configuration-service` and even change it on the fly.
> log::set_max_level

#### Access groups and tenants
After implementation of access groups we will have to alter configuration-service.
It's not clear what's exact scope of this problem is at this moment, but it's something to keep in mind.

# Test Plan
TBD

# Further considerations
## Impact on other teams
Other teams will need to redeploy their environments and pre-configure `configuration-service` with variables they were setting via `k8s`.
Currently it means that we will need to work on `DE` deployment ourselves to introduce this change, and that `NM` will need to introduce it themselves.
For any other team, helm charts provided by us should be sufficient to migrate.

## Third-party considerations
### Config file shared via network drive/ftp/...
Such config file is less elastic in terms of on the fly changes and updates.
It cannot contain logic.

### Zookeeper
It's a Java based solution. It will require substantial amount of resources to host and time to integrate.
Currently there's limited integration with rust via https://github.com/bonifaido/rust-zookeeper.
It's gonna be less flexible that homegrown solution.

## Security
There's no security risk. Configuration service is supposed to be deployed on isolated environments.
Only way to compromise this is when `API` is exposed to external party by client themself.

## Privacy considerations
No sensitive information is stored.

# Tasks and timeline
TBD
