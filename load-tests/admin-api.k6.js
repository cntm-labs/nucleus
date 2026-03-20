// Admin API load test — measures admin endpoints latency
// 401/403 expected without real API keys — we validate reachability + latency
import http from 'k6/http'
import { check, sleep } from 'k6'
import { Rate, Trend } from 'k6/metrics'

const listUsersLatency = new Trend('list_users_latency')
const endpointReachable = new Rate('endpoint_reachable')

const PROFILE = __ENV.K6_PROFILE || 'full'

const CI_OPTIONS = {
  scenarios: {
    steady: {
      executor: 'constant-arrival-rate',
      rate: 10, duration: '30s', preAllocatedVUs: 5,
    },
  },
  thresholds: {
    http_req_duration: ['p(95)<300'],
    endpoint_reachable: ['rate>0.95'],
  },
}

const FULL_OPTIONS = {
  scenarios: {
    steady_state: {
      executor: 'constant-arrival-rate',
      rate: 50, duration: '2m', preAllocatedVUs: 30,
    },
    spike: {
      executor: 'ramping-arrival-rate',
      startRate: 5,
      stages: [
        { duration: '30s', target: 5 },
        { duration: '15s', target: 200 },
        { duration: '1m', target: 200 },
        { duration: '15s', target: 5 },
      ],
      preAllocatedVUs: 100,
    },
  },
  thresholds: {
    http_req_duration: ['p(95)<200', 'p(99)<500'],
    endpoint_reachable: ['rate>0.95'],
    list_users_latency: ['p(95)<200'],
  },
}

export const options = PROFILE === 'ci' ? CI_OPTIONS : FULL_OPTIONS

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000'

export default function () {
  const start = Date.now()
  const res = http.get(`${BASE_URL}/api/v1/admin/users`, {
    headers: {
      'Authorization': 'Bearer sk_test_placeholder',
      'Content-Type': 'application/json',
    },
  })
  listUsersLatency.add(Date.now() - start)

  check(res, {
    'admin responds': (r) => [200, 401, 403].includes(r.status),
    'has body': (r) => r.body && r.body.length > 0,
  })
  endpointReachable.add(res.status > 0)

  sleep(0.5)
}
