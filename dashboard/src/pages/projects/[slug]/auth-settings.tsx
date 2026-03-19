import { useState } from 'react'

export function AuthSettingsPage() {
  const [settings, setSettings] = useState({
    password_auth: true,
    magic_link: true,
    otp: false,
    passkeys: false,
    mfa_required: false,
    session_ttl: 604800,
    jwt_lifetime: 300,
  })

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Authentication Settings</h1>

      <div className="space-y-6">
        {/* Auth methods */}
        <div className="bg-white rounded-xl border border-gray-200 p-6">
          <h2 className="text-lg font-semibold mb-4">Sign-in Methods</h2>
          <div className="space-y-3">
            {[
              { key: 'password_auth', label: 'Email + Password', desc: 'Traditional email/password authentication' },
              { key: 'magic_link', label: 'Magic Link', desc: 'Passwordless sign-in via email link' },
              { key: 'otp', label: 'One-Time Password', desc: 'SMS or email verification code' },
              { key: 'passkeys', label: 'Passkeys', desc: 'WebAuthn/FIDO2 biometric authentication' },
            ].map(method => (
              <label key={method.key} className="flex items-center justify-between p-3 rounded-lg hover:bg-gray-50">
                <div>
                  <p className="font-medium text-sm">{method.label}</p>
                  <p className="text-xs text-gray-500">{method.desc}</p>
                </div>
                <input type="checkbox" checked={settings[method.key as keyof typeof settings] as boolean}
                  onChange={e => setSettings({...settings, [method.key]: e.target.checked})}
                  className="h-4 w-4 text-nucleus-600 rounded" />
              </label>
            ))}
          </div>
        </div>

        {/* Session settings */}
        <div className="bg-white rounded-xl border border-gray-200 p-6">
          <h2 className="text-lg font-semibold mb-4">Session Configuration</h2>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium mb-1">Session TTL (seconds)</label>
              <input type="number" value={settings.session_ttl}
                onChange={e => setSettings({...settings, session_ttl: parseInt(e.target.value)})}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg" />
              <p className="text-xs text-gray-400 mt-1">{Math.floor(settings.session_ttl / 86400)} days</p>
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">JWT Lifetime (seconds)</label>
              <input type="number" value={settings.jwt_lifetime}
                onChange={e => setSettings({...settings, jwt_lifetime: parseInt(e.target.value)})}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg" />
              <p className="text-xs text-gray-400 mt-1">{settings.jwt_lifetime / 60} minutes</p>
            </div>
          </div>
        </div>

        <button className="px-6 py-2 bg-nucleus-600 text-white rounded-lg hover:bg-nucleus-700 font-medium">
          Save Changes
        </button>
      </div>
    </div>
  )
}
