# Deployment Config

> This file is read by the AI deployment pipeline. Keep it updated.
> Location: ~/.ai-studio/deploy-config.md (or wherever you want — set the path in the File Read node)

## GitHub Org
- org: `mycompany`

## Services

| Alias | Repo | Workflow | Default Env | Default Tag | Notes |
|-------|------|----------|-------------|-------------|-------|
| gateway | api-gateway | deploy.yml | production | latest | Edge proxy, deploy first |
| auth | auth-service | deploy.yml | production | latest | OAuth + JWT |
| billing | billing-service | deploy.yml | production | latest | Stripe integration |
| users | user-service | deploy.yml | production | latest | |
| notifications | notification-svc | notify-deploy.yml | production | latest | Has different workflow file |
| search | search-service | deploy.yml | production | latest | Elasticsearch dependent |
| analytics | analytics-pipeline | deploy.yml | production | latest | |
| media | media-service | deploy.yml | production | latest | S3 uploads |
| scheduler | job-scheduler | deploy.yml | production | latest | Cron jobs |
| admin | admin-dashboard | deploy-frontend.yml | production | latest | Frontend, different workflow |

## Deploy Command Pattern
```
gh workflow run {workflow} -R {org}/{repo} -f tag={tag} -f environment={env}
```

## Rules
- Default tag is `latest` unless overridden
- Default env is `production` unless overridden
- `gateway` should always deploy first (it's the edge proxy)
- `admin` is a frontend — uses `deploy-frontend.yml`, not `deploy.yml`
- `notifications` uses `notify-deploy.yml`
- Valid environments: `production`, `staging`, `dev`
- When deploying to staging, tag is usually a branch name or PR number

## Example Requests → Expected Commands
Request: "deploy everything"
→ Deploy all 10 services with tag=latest, env=production

Request: "deploy auth to v2.1.0 on staging"
→ `gh workflow run deploy.yml -R mycompany/auth-service -f tag=v2.1.0 -f environment=staging`

Request: "deploy all except billing and analytics"
→ Deploy 8 services (skip billing, analytics), tag=latest, env=production

Request: "deploy gateway and auth, auth needs v3.0.0"
→ gateway: tag=latest, env=production
→ auth: tag=v3.0.0, env=production
