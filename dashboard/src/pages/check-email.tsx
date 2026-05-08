import { Link, useSearchParams } from 'react-router-dom'

export function CheckEmailPage() {
  const [params] = useSearchParams()
  const email = params.get('email') ?? 'your inbox'

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <div className="w-full max-w-md p-8 bg-white rounded-xl shadow-sm border border-gray-200 text-center">
        <h1 className="text-2xl font-bold mb-2">Check your email</h1>
        <p className="text-gray-600 mb-4">
          We sent a verification link to <strong>{email}</strong>.
        </p>
        <p className="text-sm text-gray-500 mb-6">
          Click the link to activate your account. The link expires in 24 hours.
          Check your spam folder if you don&apos;t see it.
        </p>
        <Link to="/login" className="text-nucleus-600 hover:underline text-sm">
          Back to sign in
        </Link>
      </div>
    </div>
  )
}
