import { describe, it, expect, vi, beforeEach } from 'vitest'
import React from 'react'
import { render, screen } from '@testing-library/react'

// Mock the provider
const mockNucleusContext = {
  user: null,
  isLoaded: true,
  isSignedIn: false,
  session: null,
  organization: null,
  signOut: vi.fn(),
  getToken: vi.fn(),
  _api: {} as any,
  _sessionManager: {} as any,
  _setUser: vi.fn(),
  _setSession: vi.fn(),
  _setOrganization: vi.fn(),
}

vi.mock('../src/provider', () => ({
  useNucleus: () => mockNucleusContext,
}))

vi.mock('../src/hooks/use-sign-in', () => ({
  useSignIn: () => ({ signIn: vi.fn(), isLoading: false, error: null }),
}))

vi.mock('../src/hooks/use-sign-up', () => ({
  useSignUp: () => ({ signUp: vi.fn(), isLoading: false, error: null }),
}))

vi.mock('../src/hooks/use-oauth', () => ({
  useOAuth: () => ({ signInWithOAuth: vi.fn(), isLoading: false, error: null }),
}))

vi.mock('../src/hooks/use-mfa', () => ({
  useMfa: () => ({ verifyTotp: vi.fn(), isLoading: false, error: null }),
}))

vi.mock('../src/hooks/use-session-list', () => ({
  useSessionList: () => ({ sessions: [], fetchSessions: vi.fn(), switchSession: vi.fn() }),
}))

import { SignIn } from '../src/components/sign-in'
import { SignUp } from '../src/components/sign-up'
import { UserButton } from '../src/components/user-button'

describe('SignIn component', () => {
  it('renders email and password fields', () => {
    render(<SignIn />)
    expect(screen.getByPlaceholderText('Email')).toBeDefined()
    expect(screen.getByPlaceholderText('Password')).toBeDefined()
  })

  it('renders sign in button', () => {
    render(<SignIn />)
    const button = screen.getByRole('button', { name: 'Sign In' })
    expect(button).toBeDefined()
  })

  it('renders title', () => {
    render(<SignIn />)
    expect(screen.getByRole('heading', { name: 'Sign In' })).toBeDefined()
  })

  it('renders OAuth buttons when providers given', () => {
    render(<SignIn oauthProviders={['google', 'github']} />)
    expect(screen.getByText('Continue with Google')).toBeDefined()
    expect(screen.getByText('Continue with Github')).toBeDefined()
  })

  it('renders divider when OAuth providers present', () => {
    render(<SignIn oauthProviders={['google']} />)
    expect(screen.getByText('or')).toBeDefined()
  })

  it('does not render OAuth buttons without providers', () => {
    render(<SignIn />)
    expect(screen.queryByText(/Continue with/)).toBeNull()
  })
})

describe('SignUp component', () => {
  it('renders email and password fields', () => {
    render(<SignUp />)
    expect(screen.getByPlaceholderText('Email')).toBeDefined()
    expect(screen.getByPlaceholderText('Password')).toBeDefined()
  })

  it('renders name fields', () => {
    render(<SignUp />)
    expect(screen.getByPlaceholderText('First Name')).toBeDefined()
    expect(screen.getByPlaceholderText('Last Name')).toBeDefined()
  })

  it('renders sign up button', () => {
    render(<SignUp />)
    const button = screen.getByRole('button', { name: 'Sign Up' })
    expect(button).toBeDefined()
  })

  it('renders title', () => {
    render(<SignUp />)
    expect(screen.getByText('Create Account')).toBeDefined()
  })

  it('renders OAuth buttons when providers given', () => {
    render(<SignUp oauthProviders={['google']} />)
    expect(screen.getByText('Continue with Google')).toBeDefined()
  })
})

describe('UserButton component', () => {
  beforeEach(() => {
    mockNucleusContext.user = null
    mockNucleusContext.isSignedIn = false
  })

  it('returns null when not signed in', () => {
    const { container } = render(<UserButton />)
    expect(container.innerHTML).toBe('')
  })

  it('renders avatar button when signed in', () => {
    mockNucleusContext.user = {
      id: 'user_1',
      email: 'test@example.com',
      first_name: 'Test',
      last_name: 'User',
    } as any
    mockNucleusContext.isSignedIn = true

    render(<UserButton />)
    // Should render initials "TU"
    expect(screen.getByText('TU')).toBeDefined()
  })

  it('renders email initial when no name', () => {
    mockNucleusContext.user = {
      id: 'user_1',
      email: 'test@example.com',
      first_name: null,
      last_name: null,
    } as any
    mockNucleusContext.isSignedIn = true

    render(<UserButton />)
    expect(screen.getByText('T')).toBeDefined()
  })
})
