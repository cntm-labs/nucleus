// Token refresh load test — measures refresh endpoint latency
// 401 expected without real sessions — we validate reachability + latency
import http from 'k6/http'
import { check, sleep } from 'k6'
import { Rate, Trend } from 'k6/metrics'

const refreshLatency = new Trend('token_refresh_latency')
const endpointReachable = new Rate('endpoint_reachable')

const PROFILE = __ENV.K6_PROFILE || 'full'

const CI_OPTIONS = {
  scenarios: {
    steady: {
      executor: 'constant-arrival-rate',
      rate: 30, duration: '30s', preAllocatedVUs: 15,
    },
  },
  thresholds: {
    http_req_duration: ['p(95)<200'],
    endpoint_reachable: ['rate>0.95'],
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
    http_req_duration: ['p(95)<100', 'p(99)<300'],
    endpoint_reachable: ['rate>0.95'],
    token_refresh_latency: ['p(95)<100'],
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

  check(res, {
    'refresh responds': (r) => [200, 401, 422].includes(r.status),
  })
  endpointReachable.add(res.status > 0)

  sleep(0.3)
}
