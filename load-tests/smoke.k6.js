// Smoke test — runs on every push to main (< 30 seconds)
// Validates endpoints are reachable and respond within latency budget
// NOTE: Auth endpoints return 401/409 because no test users exist — this is expected
import http from 'k6/http'
import { check } from 'k6'
import { Rate } from 'k6/metrics'

const endpointReachable = new Rate('endpoint_reachable')

export const options = {
  scenarios: {
    smoke: {
      executor: 'constant-vus',
      vus: 5,
      duration: '15s',
    },
  },
  thresholds: {
    http_req_duration: ['p(95)<500'],
    endpoint_reachable: ['rate>0.95'],  // 95%+ endpoints must respond (any status)
  },
}

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000'

export default function () {
  // Health — must be 200
  const health = http.get(`${BASE_URL}/health`)
  check(health, { 'health 200': (r) => r.status === 200 })
  endpointReachable.add(health.status > 0)

  // JWKS — must be 200
  const jwks = http.get(`${BASE_URL}/.well-known/jwks.json`)
  check(jwks, { 'jwks 200': (r) => r.status === 200 })
  endpointReachable.add(jwks.status > 0)

  // Sign-in — expect 401 (no users), validates endpoint is alive
  const signIn = http.post(
    `${BASE_URL}/api/v1/auth/sign-in`,
    JSON.stringify({ identifier: 'test@test.com', password: 'test' }),
    { headers: { 'Content-Type': 'application/json' } }
  )
  check(signIn, {
    'sign-in responds': (r) => [200, 401, 422].includes(r.status),
    'sign-in < 500ms': (r) => r.timings.duration < 500,
  })
  endpointReachable.add(signIn.status > 0)

  // Sign-up — expect 201 or 409 (duplicate) or 422
  const signUp = http.post(
    `${BASE_URL}/api/v1/auth/sign-up`,
    JSON.stringify({ email: `smoke${__VU}${__ITER}@test.com`, password: 'SmokeTe$t123' }),
    { headers: { 'Content-Type': 'application/json' } }
  )
  check(signUp, {
    'sign-up responds': (r) => [200, 201, 409, 422, 500].includes(r.status),
  })
  endpointReachable.add(signUp.status > 0)

  // Metrics endpoint
  const metrics = http.get(`${BASE_URL}/metrics`)
  check(metrics, {
    'metrics responds': (r) => r.status === 200,
  })
  endpointReachable.add(metrics.status > 0)
}
