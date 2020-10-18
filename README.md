# HTTP static server using `Tokio` async runtime

## limited HTTP support

- requests only support `GET` endpoint, other requests return `501 unsupported`
- responses supported: `200`, `404`, `501`
- MIME types supported `html`, `text`, `json`, `binary/octetstream`
