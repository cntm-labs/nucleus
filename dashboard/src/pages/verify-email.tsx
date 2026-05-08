import { useEffect, useState } from 'react'
import { Link, useNavigate, useSearchParams } from 'react-router-dom'
import { accountApi } from '../lib/api'

type State = 'verifying' | 'success' | 'error'

export function VerifyEmailPage() {
  const [params] = useSearchParams()
  const navigate = useNavigate()
  const token = params.get('token')
  const [state, setState] = useState<State>('verifying')
  const [error, setError] = useState('')

  useEffect(() => {
    if (!token) {
      setState('error')
      setError('Missing verification token')
      return
    }
    accountApi
      .verifyEmail(token)
      .then(() => setState('success'))
      .catch((err) => {
        setState('error')
        setError(err instanceof Error ? err.message : 'Verification failed')
      })
  }, [token])

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <div className="w-full max-w-md p-8 bg-white rounded-xl shadow-sm border border-gray-200 text-center">
        {state === 'verifying' && (
          <>
            <h1 className="text-xl font-semibold mb-2">Verifying your email...</h1>
            <p className="text-gray-500 text-sm">Just a moment.</p>
          </>
        )}
        {state === 'success' && (
          <>
            <h1 className="text-2xl font-bold text-green-700 mb-2">Email verified!</h1>
            <p className="text-gray-600 mb-6">Your account is now active.</p>
            <button
              onClick={() => navigate('/login')}
              className="px-6 py-2 bg-nucleus-600 text-white rounded-lg hover:bg-nucleus-700 font-medium"
            >
              Sign in
            </button>
          </>
        )}
        {state === 'error' && (
          <>
            <h1 className="text-2xl font-bold text-red-700 mb-2">Verification failed</h1>
            <p className="text-gray-600 text-sm mb-6">{error}</p>
            <Link to="/register" className="text-nucleus-600 hover:underline">
              Try registering again
            </Link>
          </>
        )}
      </div>
    </div>
  )
}
