# Error Catalog Specification

Stable errors make downloader behavior traceable across logs, tasks, API responses, and tests.

## Error Shape

```json
{
  "code": "PIXIV_AUTH_FAILED",
  "message": "Pixiv cookie is invalid or expired.",
  "details": {}
}
```

## Core Error Codes

| Code | HTTP | User-Facing Meaning | Retry |
| --- | --- | --- | --- |
| `VALIDATION_ERROR` | 400 | Request fields are invalid | No |
| `MISSING_PIXIV_COOKIE` | 400 | Pixiv cookie is not configured | No |
| `PIXIV_AUTH_FAILED` | 401 | Pixiv cookie is invalid or expired | No |
| `PIXIV_NOT_FOUND` | 404 | Pixiv work/user/resource was not found | No |
| `PIXIV_FORBIDDEN` | 403 | Pixiv refused access to this resource | No |
| `PIXIV_RATE_LIMITED` | 429 | Pixiv is rate limiting requests | Yes, limited |
| `PIXIV_NETWORK_ERROR` | 502 | Network request to Pixiv failed | Yes |
| `PIXIV_PARSE_ERROR` | 502 | Pixiv response could not be parsed | No |
| `R18_POLICY_SKIPPED` | 200 item skip | Item skipped by R18 policy | No |
| `FILESYSTEM_WRITE_FAILED` | 500 | Could not write downloaded file | Maybe |
| `FILESYSTEM_PATH_COLLISION` | 500 | Planned path conflicts with another image | No |
| `SQLITE_ERROR` | 500 | Local database operation failed | Maybe |
| `AI_CONFIG_MISSING` | 400 | DeepSeek key or base URL missing | No |
| `AI_PARSE_FAILED` | 502 | AI result was not valid structured output | No |
| `TASK_CANCELLED` | 200/409 | Task was cancelled | No |
| `TASK_NOT_FOUND` | 404 | Requested task does not exist | No |
| `INTERNAL_ERROR` | 500 | Unexpected backend failure | Maybe |

## Logging Rules

- Never log full `PHPSESSID`.
- Never log full DeepSeek API key.
- Log Pixiv ID, task ID, page index, and phase whenever available.
- Include error code in task logs and API error responses.

## Task Status Mapping

| Error Scenario | Task Status |
| --- | --- |
| Request invalid before task creation | No task |
| Missing cookie before task creation | No task |
| Auth failure after task creation | `failed` |
| Single item not found | `failed` |
| One item fails in batch | `completed_with_errors` |
| All batch items skipped by policy | `completed` with logs |
| Filesystem write failure in single task | `failed` |
| Filesystem write failure in batch | `completed_with_errors` or `failed` if systemic |
