# Front Matter

```
Title: Configuration service
Author: Łukasz Biel
Team: CDL
Reviewer: CDLTeam
Created on: 5/2/2021
Last updated: 5/2/2021
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


Structure assumes existence of `data-materializer`, and when `data-materializer` would be introduced this structure can change.
`repository_type` refers to distinction between `TIMESERIES` and `DOCUMENT_STORAGE`, 
and is necessary when validating payloads, as different kinds of repositories have slightly different message formats.

### Configuration Service
`configuration-service` would replace `SR` functionality of serving repository metadata.

All `command-service`s would need to receive additional config: `CONFIGURATION_SERVICE_URL`.
Once started `CS` would send a request to `configuration-service` with it's metadata in order to `register`. Such request would consist of:
* repository database - a string
* repository type - a string/enum
* repository_id - a uuid
A command service would start listening on port/topic/etc. provided in response to register request.

For each repository registration request `configuration-service` would add an entry to it's DB.
For each `repository_id` `configuration-service` stores:
* human readable name (eg. postgres-document-1)
* ingestion means (address/topic/queue)
* egestion address
* repository type
* repository database

`configuration-service` allows users to declare `schema_id`s connected to given repository.
Such connection dictionary would be keyed on `schema_id` to decrease lookup latency
and would require connection between `schema` and `repository` to be many-to-one.

All configuration in `configuration-service` should be backed by some database (we can consider sled here, however we should implemented 
a wrapper over it, so features like replication are shared between `configuration-service` and `schema-registry`).
It needs to support replication and disk backups.

`configuration-service` should support `--import` and `--export` flags as current `schema-registry` does.

All communication with `configuration-service` should be done via `gRPC`,
while `API` crate should provide end-user, easy to read/write, http api access.

Supported `gRPC` queries are:

* register
* list_repositories
* describe_repository
* add_repository
* rm_repository
* update_repository_X, where X is a repository's field
* add_schema
* rm_schema
* get_all_repository_schemas

### Questions

* Should we store repository database in `configuration-service` since it's only used in repo name creation?
* Should `configuration-service` perform a health check on repos to determine if one wasn't deleted? 
* Should it verify existence and manage creation of topics/queues for `command-service`s when cdl uses `MQ`?
* We must consider whether `repository_id` should be a string like kafka `group_id` or uuid.
* What should be behaviour of register when repository wasn't pre-declared in configuration service?
* Should it be `CS` responsibility to connect to repository or should repository contain special connector service?
* Shouldn't `QS` also get info about configuration and eg. port from `configuration-service`?

### Other considerations
#### Connector
We could write `configuration-service` as a connector, not a database. In such case CS would have eg. topic configuration provided via envs and
it would send these to `configuration-service`. `configuration-service` then would serve this metadata to DR and QR.

#### Propagate protocol via configuration service
`configuration-service` could be set up with a variable denoting protocol used in `CDL` deployment and serve it to repositories,
instead of this protocol being served to them via an env.

#### A library
We could write `configuration-service` in such way so that DR/CDLite/our clients can include it as an library.

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
