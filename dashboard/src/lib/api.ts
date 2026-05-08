const API_BASE = '/api/v1/dashboard'
const TOKEN_KEY = 'nucleus_dashboard_token'

class ApiClient {
  private token: string | null = null

  constructor() {
    this.token = localStorage.getItem(TOKEN_KEY)
  }

  setToken(token: string | null) {
    this.token = token
    if (token) {
      localStorage.setItem(TOKEN_KEY, token)
    } else {
      localStorage.removeItem(TOKEN_KEY)
    }
  }

  private async request<T>(path: string, options: RequestInit = {}): Promise<T> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...((options.headers as Record<string, string>) || {}),
    }

    if (this.token) {
      headers['Authorization'] = `Bearer ${this.token}`
    }

    const res = await fetch(`${API_BASE}${path}`, {
      ...options,
      headers,
    })

    if (!res.ok) {
      const error = await res.json().catch(() => ({ error: { message: res.statusText } }))
      throw new ApiError(res.status, error.error?.code || 'unknown', error.error?.message || res.statusText)
    }

    return res.json()
  }

  get<T>(path: string) { return this.request<T>(path) }
  post<T>(path: string, body?: unknown) { return this.request<T>(path, { method: 'POST', body: body ? JSON.stringify(body) : undefined }) }
  patch<T>(path: string, body: unknown) { return this.request<T>(path, { method: 'PATCH', body: JSON.stringify(body) }) }
  delete<T>(path: string) { return this.request<T>(path, { method: 'DELETE' }) }
}

export class ApiError extends Error {
  constructor(public status: number, public code: string, message: string) {
    super(message)
  }
}

export const api = new ApiClient()

// ---------------------------------------------------------------------------
// Typed account auth helpers
// ---------------------------------------------------------------------------

export interface Account {
  id: string
  email: string
  name: string
  company: string | null
  is_active: boolean
  email_verified: boolean
  created_at: string
  updated_at: string
}

export interface SignUpResponse {
  account: Account
  message: string
}

export interface SignInResponse {
  account: Account
  token: string
}

export interface VerifyEmailResponse {
  account: Account
}

export const accountApi = {
  signUp: (email: string, password: string, name: string, company?: string) =>
    api.post<SignUpResponse>('/auth/sign-up', { email, password, name, company }),

  signIn: (email: string, password: string) =>
    api.post<SignInResponse>('/auth/sign-in', { email, password }),

  verifyEmail: (token: string) =>
    api.post<VerifyEmailResponse>('/auth/verify-email', { token }),

  me: () =>
    api.get<Account>('/auth/me'),

  signOut: () =>
    api.post<{ success: boolean }>('/auth/sign-out'),
}

// ---------------------------------------------------------------------------
// Project helpers
// ---------------------------------------------------------------------------

export interface Project {
  id: string
  account_id: string
  name: string
  slug: string
  data_mode: string
  environment: string
  created_at: string
  updated_at: string
}

export interface ProjectListResponse {
  data: Project[]
  has_more: boolean
  next_cursor: string | null
}

export const projectsApi = {
  list: () => api.get<ProjectListResponse>('/projects'),
  create: (data: { name: string; slug: string; data_mode?: string }) =>
    api.post<Project>('/projects', data),
  get: (id: string) => api.get<Project>(`/projects/${id}`),
}
