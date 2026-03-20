import http from 'k6/http'
import { check, sleep } from 'k6'
import { Rate, Trend } from 'k6/metrics'

const refreshLatency = new Trend('token_refresh_latency')
const refreshFailRate = new Rate('token_refresh_fail_rate')

export const options = {
  scenarios: {
    steady_state: {
      executor: 'constant-arrival-rate',
      rate: 200, duration: '5m', preAllocatedVUs: 100,
    },
    spike: {
      executor: 'ramping-arrival-rate',
      startRate: 20,
      stages: [
        { duration: '1m', target: 20 },
        { duration: '30s', target: 1000 },
        { duration: '2m', target: 1000 },
        { duration: '30s', target: 20 },
      ],
      preAllocatedVUs: 300,
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<50', 'p(99)<150'],
    'token_refresh_latency': ['p(95)<50'],
    'token_refresh_fail_rate': ['rate<0.01'],
    'http_req_failed': ['rate<0.01'],
  },
}

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000'

export function setup() {
  // Acquire a valid refresh token via sign-in
  const res = http.post(`${BASE_URL}/api/v1/auth/sign-in`, JSON.stringify({
    identifier: 'loadtest@test.com',
    password: 'TestPass123!',
  }), { headers: { 'Content-Type': 'application/json' } })

  if (res.status === 200) {
    const body = JSON.parse(res.body)
    return { refreshToken: body.refresh_token }
  }
  return { refreshToken: 'placeholder_refresh_token' }
}

export default function (data) {
  const start = Date.now()
  const res = http.post(`${BASE_URL}/api/v1/auth/token/refresh`, JSON.stringify({
    refresh_token: data.refreshToken,
  }), { headers: { 'Content-Type': 'application/json' } })
  refreshLatency.add(Date.now() - start)
  refreshFailRate.add(res.status !== 200)

  check(res, {
    'refresh status 200 or 401': (r) => r.status === 200 || r.status === 401,
    'latency < 50ms': (r) => r.timings.duration < 50,
  })

  // If we got a new refresh token, use it for subsequent requests
  if (res.status === 200) {
    try {
      const body = JSON.parse(res.body)
      if (body.refresh_token) {
        data.refreshToken = body.refresh_token
      }
    } catch (_) {}
  }

  sleep(0.5)
}
