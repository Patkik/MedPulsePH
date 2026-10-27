# 22. Browser, PWA, API, and abuse hardening

**Status:** In progress; `backend/src/hardening.rs` attaches mandatory HTTP response security headers (`Cache-Control: no-store, no-cache, must-revalidate`, `Content-Security-Policy`, `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`) to `router()`, redacts sensitive tap query parameters (`redact_sensitive_url_params`), enforces outbound HTTPS + SSRF destination checks (`validate_outbound_https_destination`), and constructs hospital-aware composite rate-limit keys (`CompositeRateLimitKey`).

[Plan index](../../IMPLEMENTATION_PLAN.md) | [Iteration packages](ITERATIONS.md) | Primary package: I09

---

### Browser and delivery controls

- Use a defined origin layout. Keep public emergency content and staff sessions isolated where feasible, with host-scoped cookies and deliberately scoped service workers. Origin isolation should not be claimed if only paths differ.
- Set HTTPS and appropriate HSTS, Content Security Policy, anti-framing controls, referrer policy, and permissions policy. Stage CSP changes and monitor violations without leaking patient payloads.
- Escape clinical and patient-entered text; sanitize imported FHIR XHTML/narrative with an allowlist. Do not render untrusted HTML or permit an imported document to execute in the authenticated application origin.
- Redact tap URL parameters at every ingress, proxy, CDN, tracing, and error-reporting boundary. Exchange an accepted proof for a short-lived context and navigate to a clean URL. Cleaning browser history does not erase an already recorded upstream log.
- Exclude protected data and tap URLs from HTML pre-rendering, generated sitemaps, CDN caches, link previews, search indexing, telemetry, and build artifacts. `robots.txt` and `noindex` are not authorization controls.
- Test browser back/forward cache, page restoration, suspended tabs, PWA relaunch, logout across tabs, and device handover. Keep protected UI hidden until revalidation on resume. Clearing application state cannot erase an external screenshot or compromised browser.
- Define service-worker scope, asset versioning, stale-client/API compatibility, update rollback, and an emergency worker-disable procedure. A code rollback must account for a worker continuing to serve old assets.
- Avoid unnecessary third-party scripts on clinical, emergency, login, and callback pages. No session replay or input capture of clinical screens. Minimize third-party fonts and runtime resources through controlled hosting.

### API and resource limits

Apply strict request/response schemas and field allowlists; reject mass assignment of roles, patient links, consent state, source organization, or finalization flags. Bound body size, nesting, string length, pagination, search complexity, export size, connection lifetime, and concurrency. Validate content types and use parameterized SQL. Keep client-facing errors safe and make correlation references usable by support without revealing records.

Protect expensive PIN hashing, KMS calls, uploads, search, export generation, and OTP delivery with layered per-actor/per-context and infrastructure limits. Do not rely on IP limits alone in hospitals sharing an egress address. Prevent enumeration and balance throttling with legitimate emergency access; define escalation and recovery from malicious lockout attempts. Separate emergency-path capacity from costly administrative/clinical jobs without giving invalid proofs a bypass.

Specify deadlines, cancellation, bounded queues, circuit breakers, database pool limits, and backpressure. Never use unbounded spawned tasks or retries. Track resource and supplier costs as well as request counts. Reference: [OWASP API resource consumption](https://api-security.owasp.org/editions/2023/en/0xa4-unrestricted-resource-consumption/).

Allowlist outbound integration destinations; block server-side requests to private, link-local, metadata-service, and other unintended addresses, including redirect and DNS-rebinding cases. Do not let arbitrary FHIR references, image URLs, webhooks, or token key URLs turn the API into a fetch proxy. CORS is a browser restriction, not an API authorization mechanism.
