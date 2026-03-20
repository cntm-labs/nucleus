// Smoke test — runs on every push to main (< 30 seconds)
// Validates endpoints are reachable and respond within budget
import http from 'k6/http'
import { check } from 'k6'

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
    http_req_failed: ['rate<0.05'],
  },
}

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000'

export default function () {
  // Health
  const health = http.get(`${BASE_URL}/health`)
  check(health, { 'health 200': (r) => r.status === 200 })

  // JWKS
  const jwks = http.get(`${BASE_URL}/.well-known/jwks.json`)
  check(jwks, { 'jwks 200': (r) => r.status === 200 })

  // Sign-in (expect 401 — no real users, but endpoint responds)
  const signIn = http.post(
    `${BASE_URL}/api/v1/auth/sign-in`,
    JSON.stringify({ identifier: 'test@test.com', password: 'test' }),
    { headers: { 'Content-Type': 'application/json' } }
  )
  check(signIn, {
    'sign-in responds': (r) => r.status === 401 || r.status === 200,
    'sign-in < 500ms': (r) => r.timings.duration < 500,
  })

  // Sign-up
  const signUp = http.post(
    `${BASE_URL}/api/v1/auth/sign-up`,
    JSON.stringify({ email: `smoke${__VU}${__ITER}@test.com`, password: 'SmokeTe$t123' }),
    { headers: { 'Content-Type': 'application/json' } }
  )
  check(signUp, {
    'sign-up responds': (r) => [200, 201, 409].includes(r.status),
  })
}
