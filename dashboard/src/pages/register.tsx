import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { accountApi } from '../lib/api'

export function RegisterPage() {
  const navigate = useNavigate()
  const [form, setForm] = useState({ name: '', email: '', password: '', company: '' })
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoading(true)
    setError('')
    try {
      await accountApi.signUp(
        form.email,
        form.password,
        form.name,
        form.company || undefined,
      )
      navigate(`/check-email?email=${encodeURIComponent(form.email)}`)
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Registration failed'
      setError(message)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <div className="w-full max-w-md p-8 bg-white rounded-xl shadow-sm border border-gray-200">
        <h1 className="text-2xl font-bold text-center mb-2">Create Account</h1>
        <p className="text-gray-500 text-center mb-8">Start building with Nucleus</p>
        {error && <div className="bg-red-50 text-red-600 text-sm p-3 rounded-lg mb-4">{error}</div>}
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label htmlFor="register-name" className="block text-sm font-medium text-gray-700 mb-1">Full Name</label>
            <input id="register-name" type="text" value={form.name} onChange={e => setForm({...form, name: e.target.value})}
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-nucleus-500 focus:border-transparent outline-none" required />
          </div>
          <div>
            <label htmlFor="register-email" className="block text-sm font-medium text-gray-700 mb-1">Email</label>
            <input id="register-email" type="email" value={form.email} onChange={e => setForm({...form, email: e.target.value})}
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-nucleus-500 focus:border-transparent outline-none" required />
          </div>
          <div>
            <label htmlFor="register-company" className="block text-sm font-medium text-gray-700 mb-1">Company (optional)</label>
            <input id="register-company" type="text" value={form.company} onChange={e => setForm({...form, company: e.target.value})}
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-nucleus-500 focus:border-transparent outline-none" />
          </div>
          <div>
            <label htmlFor="register-password" className="block text-sm font-medium text-gray-700 mb-1">Password</label>
            <input id="register-password" type="password" value={form.password} onChange={e => setForm({...form, password: e.target.value})}
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-nucleus-500 focus:border-transparent outline-none" required minLength={8} />
            <p className="text-xs text-gray-400 mt-1">Minimum 8 characters</p>
          </div>
          <button type="submit" disabled={loading}
            className="w-full py-2 bg-nucleus-600 text-white rounded-lg hover:bg-nucleus-700 font-medium disabled:opacity-50">
            {loading ? 'Creating account...' : 'Create Account'}
          </button>
        </form>
        <p className="text-center text-sm text-gray-500 mt-6">
          Already have an account? <Link to="/login" className="text-nucleus-600 hover:underline">Sign in</Link>
        </p>
      </div>
    </div>
  )
}
