// Auth flow load test — validates auth endpoints respond under load
// NOTE: Without seeded test users, auth endpoints return 401 — we measure
// latency and reachability, not success rate
import http from 'k6/http'
import { check, sleep } from 'k6'
import { Rate, Trend } from 'k6/metrics'

const signInLatency = new Trend('sign_in_latency')
const endpointReachable = new Rate('endpoint_reachable')

const PROFILE = __ENV.K6_PROFILE || 'full'

const CI_OPTIONS = {
  scenarios: {
    steady: {
      executor: 'constant-arrival-rate',
      rate: 20, duration: '30s', preAllocatedVUs: 10,
    },
  },
  thresholds: {
    http_req_duration: ['p(95)<500'],
    endpoint_reachable: ['rate>0.95'],
  },
}

const FULL_OPTIONS = {
  scenarios: {
    steady_state: {
      executor: 'constant-arrival-rate',
      rate: 100, duration: '2m', preAllocatedVUs: 50,
    },
    spike: {
      executor: 'ramping-arrival-rate',
      startRate: 10,
      stages: [
        { duration: '30s', target: 10 },
        { duration: '15s', target: 300 },
        { duration: '1m', target: 300 },
        { duration: '15s', target: 10 },
      ],
      preAllocatedVUs: 150,
    },
  },
  thresholds: {
    http_req_duration: ['p(95)<300', 'p(99)<1000'],
    endpoint_reachable: ['rate>0.95'],
    sign_in_latency: ['p(95)<300'],
  },
}

export const options = PROFILE === 'ci' ? CI_OPTIONS : FULL_OPTIONS

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000'

export default function () {
  // Health check — must succeed
  const healthRes = http.get(`${BASE_URL}/health`)
  check(healthRes, { 'health 200': (r) => r.status === 200 })
  endpointReachable.add(healthRes.status > 0)

  // Sign-in — measure latency (401 expected without test users)
  const start = Date.now()
  const signInRes = http.post(`${BASE_URL}/api/v1/auth/sign-in`, JSON.stringify({
    identifier: `user${__VU}@test.com`,
    password: 'TestPass123!',
  }), { headers: { 'Content-Type': 'application/json' } })
  signInLatency.add(Date.now() - start)

  check(signInRes, {
    'sign-in responds': (r) => [200, 401, 422].includes(r.status),
    'latency < 500ms': (r) => r.timings.duration < 500,
  })
  endpointReachable.add(signInRes.status > 0)

  sleep(0.5)
}
