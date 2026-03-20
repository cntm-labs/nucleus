// Auth flow load test — runs weekly or manually
import http from 'k6/http'
import { check, sleep } from 'k6'
import { Rate, Trend } from 'k6/metrics'

const signInLatency = new Trend('sign_in_latency')
const signInFailRate = new Rate('sign_in_fail_rate')

// Duration controlled by K6_PROFILE env: "ci" = short, default = full
const PROFILE = __ENV.K6_PROFILE || 'full'

const CI_OPTIONS = {
  scenarios: {
    steady: {
      executor: 'constant-arrival-rate',
      rate: 20, duration: '30s', preAllocatedVUs: 10,
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<500'],
    'http_req_failed': ['rate<0.05'],
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
    'http_req_duration': ['p(95)<200', 'p(99)<500'],
    'sign_in_latency': ['p(95)<150'],
    'sign_in_fail_rate': ['rate<0.01'],
    'http_req_failed': ['rate<0.01'],
  },
}

export const options = PROFILE === 'ci' ? CI_OPTIONS : FULL_OPTIONS

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000'

export default function () {
  const healthRes = http.get(`${BASE_URL}/health`)
  check(healthRes, { 'health 200': (r) => r.status === 200 })

  const start = Date.now()
  const signInRes = http.post(`${BASE_URL}/api/v1/auth/sign-in`, JSON.stringify({
    identifier: `user${__VU}@test.com`,
    password: 'TestPass123!',
  }), { headers: { 'Content-Type': 'application/json' } })
  signInLatency.add(Date.now() - start)
  signInFailRate.add(signInRes.status !== 200)

  check(signInRes, {
    'sign-in responds': (r) => r.status === 200 || r.status === 401,
    'latency < 500ms': (r) => r.timings.duration < 500,
  })

  sleep(0.5)
}
