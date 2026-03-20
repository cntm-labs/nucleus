import http from 'k6/http'
import { check, sleep } from 'k6'
import { Rate, Trend } from 'k6/metrics'

const listUsersLatency = new Trend('list_users_latency')
const listUsersFailRate = new Rate('list_users_fail_rate')

export const options = {
  scenarios: {
    steady_state: {
      executor: 'constant-arrival-rate',
      rate: 50, duration: '5m', preAllocatedVUs: 30,
    },
    spike: {
      executor: 'ramping-arrival-rate',
      startRate: 5,
      stages: [
        { duration: '1m', target: 5 },
        { duration: '30s', target: 200 },
        { duration: '2m', target: 200 },
        { duration: '30s', target: 5 },
      ],
      preAllocatedVUs: 100,
    },
  },
  thresholds: {
    'http_req_duration': ['p(95)<100', 'p(99)<300'],
    'list_users_latency': ['p(95)<100'],
    'list_users_fail_rate': ['rate<0.01'],
    'http_req_failed': ['rate<0.01'],
  },
}

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000'
const ADMIN_API_KEY = __ENV.ADMIN_API_KEY || 'test_admin_api_key'

export default function () {
  const start = Date.now()
  const res = http.get(`${BASE_URL}/api/v1/admin/users`, {
    headers: {
      'Authorization': `Bearer ${ADMIN_API_KEY}`,
      'Content-Type': 'application/json',
    },
  })
  listUsersLatency.add(Date.now() - start)
  listUsersFailRate.add(res.status !== 200)

  check(res, {
    'list users status 200 or 403': (r) => r.status === 200 || r.status === 403,
    'latency < 100ms': (r) => r.timings.duration < 100,
    'response has body': (r) => r.body && r.body.length > 0,
  })

  // Paginated request
  const pageRes = http.get(`${BASE_URL}/api/v1/admin/users?page=1&per_page=10`, {
    headers: {
      'Authorization': `Bearer ${ADMIN_API_KEY}`,
      'Content-Type': 'application/json',
    },
  })

  check(pageRes, {
    'paginated request succeeds': (r) => r.status === 200 || r.status === 403,
  })

  sleep(1)
}
