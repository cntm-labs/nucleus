import http from 'k6/http'
import { check, sleep } from 'k6'
import { Rate, Trend } from 'k6/metrics'

const signInLatency = new Trend('sign_in_latency')
const signInFailRate = new Rate('sign_in_fail_rate')

export const options = {
  scenarios: {
    steady_state: {
      executor: 'constant-arrival-rate',
      rate: 100, duration: '5m', preAllocatedVUs: 50,
    },
    spike: {
      executor: 'ramping-arrival-rate',
      startRate: 10,
      stages: [
        { duration: '1m', target: 10 },
        { duration: '30s', target: 500 },
        { duration: '2m', target: 500 },
        { duration: '30s', target: 10 },
      ],
      preAllocatedVUs: 200,
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<200', 'p(99)<500'],
    'sign_in_latency': ['p(95)<150'],
    'sign_in_fail_rate': ['rate<0.01'],
    'http_req_failed': ['rate<0.01'],
  },
}

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000'

export default function () {
  // Health check
  const healthRes = http.get(`${BASE_URL}/health`)
  check(healthRes, { 'health 200': (r) => r.status === 200 })

  // Sign-in flow
  const start = Date.now()
  const signInRes = http.post(`${BASE_URL}/api/v1/auth/sign-in`, JSON.stringify({
    identifier: `user${__VU}@test.com`,
    password: 'TestPass123!',
  }), { headers: { 'Content-Type': 'application/json' } })
  signInLatency.add(Date.now() - start)
  signInFailRate.add(signInRes.status !== 200)

  check(signInRes, {
    'sign-in status 200 or 401': (r) => r.status === 200 || r.status === 401,
    'latency < 200ms': (r) => r.timings.duration < 200,
  })

  sleep(1)
}
