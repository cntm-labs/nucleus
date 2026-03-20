// Token refresh load test — runs weekly or manually
import http from 'k6/http'
import { check, sleep } from 'k6'
import { Rate, Trend } from 'k6/metrics'

const refreshLatency = new Trend('token_refresh_latency')
const refreshFailRate = new Rate('token_refresh_fail_rate')

const PROFILE = __ENV.K6_PROFILE || 'full'

const CI_OPTIONS = {
  scenarios: {
    steady: {
      executor: 'constant-arrival-rate',
      rate: 30, duration: '30s', preAllocatedVUs: 15,
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<200'],
    'http_req_failed': ['rate<0.05'],
  },
}

const FULL_OPTIONS = {
  scenarios: {
    steady_state: {
      executor: 'constant-arrival-rate',
      rate: 200, duration: '2m', preAllocatedVUs: 100,
    },
    spike: {
      executor: 'ramping-arrival-rate',
      startRate: 20,
      stages: [
        { duration: '30s', target: 20 },
        { duration: '15s', target: 500 },
        { duration: '1m', target: 500 },
        { duration: '15s', target: 20 },
      ],
      preAllocatedVUs: 200,
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<50', 'p(99)<150'],
    'token_refresh_latency': ['p(95)<50'],
    'token_refresh_fail_rate': ['rate<0.01'],
    'http_req_failed': ['rate<0.01'],
  },
}

export const options = PROFILE === 'ci' ? CI_OPTIONS : FULL_OPTIONS

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000'

export default function () {
  const start = Date.now()
  const res = http.post(`${BASE_URL}/api/v1/auth/token/refresh`, JSON.stringify({
    session_id: 'test_session',
  }), { headers: { 'Content-Type': 'application/json' } })
  refreshLatency.add(Date.now() - start)
  refreshFailRate.add(res.status !== 200)

  check(res, {
    'refresh responds': (r) => r.status === 200 || r.status === 401,
  })

  sleep(0.3)
}
