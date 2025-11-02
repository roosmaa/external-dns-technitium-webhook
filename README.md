# external-dns-technitium-webhook

> [!WARNING]
> This is homelab quality software, and not meant for production usage. You have been warned.

External-dns-technitium-webhook is an [ExternalDNS](https://kubernetes-sigs.github.io/external-dns/latest/) webhook to
integrate it with [Technitium DNS](https://technitium.com/dns/).

## Usage

The application expects all configuration to be passed in via environment variables.

| Environment Variable  | Description                                                                                               |
|-----------------------|-----------------------------------------------------------------------------------------------------------|
| `LISTEN_ADDRESS`      | The address the webhook server binds to (defaults to `0.0.0.0`).                                          |
| `LISTEN_PORT`         | The port the webhook server listens ono (defaults to `3000`).                                             |
| `TECHNITIUM_URL`      | The URL of the Technitium DNS server (required).                                                          |
| `TECHNITIUM_USERNAME` | The username to authenticate with the Technitium DNS server (required).                                   |
| `TECHNITIUM_PASSWORD` | The password for the user. Required when `TECHNITIUM_TOKEN` is not supplied.                              |
| `TECHNITIUM_TOKEN`    | A pre-generated Technitium API token. When set, the webhook skips password login and reuses this token.   |
| `ZONE`                | The zone to manage (e.g. `example.com`, required).                                                        |
| `DOMAIN_FILTERS`      | A semicolon-separated list of domain filters to apply (e.g. `foo.example.com;bar.example.com`, optional). |

Provide either `TECHNITIUM_PASSWORD` *or* `TECHNITIUM_TOKEN`. When a token is supplied, the webhook uses it directly and does not attempt to refresh credentials via the login endpoint.

### Zone Handling

If the specified `ZONE` doesn't exist in Technitium DNS, it will be created automatically when the application starts.

The zone created will be of Forward type, with forwarder to `this-server` and DNSSEC validation enabled. This means
that if the record doesn't exist in the zone on Technitium DNS, the internal resolver will be used and the DNS servers
on the internet will be consulted.

## Example Kubernetes Deployment

When deploying on kubernetes, the Technitium DNS webhook can be deployed as a sidecar to the external-dns deployment.

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: external-dns-technitium-dns
  namespace: external-dns
  labels:
    app.kubernetes.io/name: external-dns
spec:
  strategy:
    type: Recreate
  selector:
    matchLabels:
      app.kubernetes.io/name: external-dns
  template:
    metadata:
      labels:
        app.kubernetes.io/name: external-dns
    spec:
      serviceAccountName: external-dns
      containers:
        - name: external-dns
          image: registry.k8s.io/external-dns/external-dns
          args:
            - --source=service
            - --source=ingress
            - --registry=noop
            - --provider=webhook
            - --webhook-provider-url=http://localhost:5580
        - name: webhook
          image: ghcr.io/roosmaa/external-dns-technitium-webhook
          env:
            - name: RUST_LOG
              value: "external_dns_technitium_webhook=info"
            - name: LISTEN_PORT
              value: "5580"
            - name: TECHNITIUM_URL
              value: "http://technitium-dns-dashboard.dns.svc.cluster.local:5380"
            - name: ZONE
              value: "example.com"
          envFrom:
            - secretRef:
                name: technitium-dns
          resources:
            requests:
              cpu: 1m
              memory: 10Mi
          readinessProbe:
            httpGet:
              port: 5580
              path: /health
            failureThreshold: 1
---
kind: Secret
type: Opaque
apiVersion: v1
stringData:
  TECHNITIUM_USERNAME: admin
  # Provide exactly one of the following:
  TECHNITIUM_PASSWORD: example-password
  # TECHNITIUM_TOKEN: example-static-token
metadata:
  name: technitium-dns
  namespace: external-dns
```
